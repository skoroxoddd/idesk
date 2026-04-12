use crate::capture::capturer::Capturer;
use crate::capture::frame::CaptureFrame;
use crate::error::{AppError, Result};
use x11rb::connection::Connection;
use x11rb::protocol::shm::{self, ConnectionExt as _};
use x11rb::protocol::xproto::{self, ConnectionExt as _};
use x11rb::rust_connection::RustConnection;

const SHMAT_FAILED: *mut libc::c_void = usize::MAX as *mut libc::c_void;

pub struct X11Capturer {
    conn: RustConnection,
    screen: xproto::Screen,
    shmseg: shm::Seg,
    shmid: i32,
    shm_size: usize,
    width: u16,
    height: u16,
}

impl X11Capturer {
    pub fn new() -> Result<Self> {
        let (conn, screen_num) =
            x11rb::connect(None).map_err(|e| AppError::Capture(format!("Failed to connect to X11: {e}")))?;

        let screen = conn.setup().roots[screen_num].clone();

        let width = screen.width_in_pixels;
        let height = screen.height_in_pixels;
        let stride = width * 4;
        let shm_size = (stride * height) as usize;

        // Create shared memory segment
        let shmid = unsafe { libc::shmget(libc::IPC_PRIVATE, shm_size, libc::IPC_CREAT | 0o600) };
        if shmid < 0 {
            return Err(AppError::Capture(format!("shmget failed: {}", std::io::Error::last_os_error())));
        }

        let shm_data = unsafe { libc::shmat(shmid, std::ptr::null(), 0) };
        if shm_data == SHMAT_FAILED {
            unsafe { libc::shmctl(shmid, libc::IPC_RMID, std::ptr::null_mut()) };
            return Err(AppError::Capture(format!("shmat failed: {}", std::io::Error::last_os_error())));
        }

        // Detach immediately — we'll re-attach on each capture
        unsafe { libc::shmdt(shm_data) };

        let shmseg = conn.generate_id().map_err(|e| AppError::Capture(format!("Failed to generate X11 ID: {e}")))?;
        shm::attach(&conn, shmseg, shmid as u32, true)
            .map_err(|e| AppError::Capture(format!("XShm attach failed: {e}")))?;

        // Sync to ensure SHM is attached
        let _ = xproto::get_input_focus(&conn)
            .map_err(|e| AppError::Capture(format!("X11 sync failed: {e}")))?
            .reply()
            .map_err(|e| AppError::Capture(format!("X11 sync reply failed: {e}")))?;

        Ok(Self {
            conn,
            screen,
            shmseg,
            shmid,
            shm_size,
            width,
            height,
        })
    }
}

impl Capturer for X11Capturer {
    fn start(&mut self) -> Result<()> {
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        Ok(())
    }

    fn capture_frame(&mut self) -> Result<CaptureFrame> {
        let stride = self.width as u32 * 4;

        // Get image via XShm
        let cookie = shm::get_image(
            &self.conn,
            self.screen.root,
            0, 0,
            self.width, self.height,
            !0u32,
            xproto::ImageFormat::Z_PIXMAP.into(),
            self.shmseg,
            0,
        ).map_err(|e| AppError::Capture(format!("XShm get_image failed: {e}")))?;

        let reply = cookie.reply()
            .map_err(|e| AppError::Capture(format!("XShm get_image reply failed: {e}")))?;

        // Read from shared memory
        let shm_data = unsafe { libc::shmat(self.shmid, std::ptr::null(), 0) };
        if shm_data == SHMAT_FAILED {
            return Err(AppError::Capture("shmat failed during capture".to_string()));
        }

        let data = unsafe { std::slice::from_raw_parts(shm_data as *const u8, self.shm_size) }.to_vec();
        unsafe { libc::shmdt(shm_data) };

        Ok(CaptureFrame {
            data,
            width: self.width as u32,
            height: self.height as u32,
            stride,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        })
    }

    fn width(&self) -> u32 {
        self.width as u32
    }

    fn height(&self) -> u32 {
        self.height as u32
    }
}

impl Drop for X11Capturer {
    fn drop(&mut self) {
        unsafe { libc::shmctl(self.shmid, libc::IPC_RMID, std::ptr::null_mut()) };
    }
}
