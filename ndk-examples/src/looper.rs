//! Demonstrates how to manage application lifetime using Android's `Looper`
//! 演示如何使用 Android 的“Looper”管理应用程序生命周期

use {
    android_logger::{Config, init_once},
    log::{LevelFilter, error, info},
    ndk::looper::{FdEvent, ThreadLooper},
    std::{
        mem::MaybeUninit,
        os::{
            fd::{AsRawFd, BorrowedFd},
            unix::prelude::RawFd,
        },
        thread::{sleep, spawn},
        time::Duration,
    },
    winit::platform::android::activity::{AndroidApp, InputStatus, MainEvent, PollEvent},
};

const U32_SIZE: usize = size_of::<u32>();

//noinspection SpellCheckingInspection
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    init_once(Config::default().with_max_level(LevelFilter::Info));

    // Retrieve the Looper that android-activity created for us on the current thread.
    // android-activity uses this to block on events and poll file descriptors with a single mechanism.
    // 检索 android-activity 在当前线程上为我们创建的 Looper。android-activity 使用它来阻止事件
    // 并通过单一机制轮询文件描述符。
    let looper =
        ThreadLooper::for_thread().expect("ndk-glue did not attach thread looper before main()!");

    fn create_pipe() -> [RawFd; 2] {
        let mut ends = MaybeUninit::<[RawFd; 2]>::uninit();
        assert_eq!(unsafe { libc::pipe(ends.as_mut_ptr().cast()) }, 0);
        unsafe { ends.assume_init() }
    }

    // Create a Unix pipe to send custom events to the Looper. ndk-glue uses a similar mechanism to deliver
    // ANativeActivityCallbacks asynchronously to the Looper through NDK_GLUE_LOOPER_EVENT_PIPE_IDENT.
    // 创建一个 Unix 管道，将自定义事件发送到 Looper。ndk-glue 使用类似的机制，通过
    // NDK_GLUE_LOOPER_EVENT_PIPE_IDENT 将 ANativeActivityCallbacks 异步传递给 Looper。
    let custom_event_pipe = create_pipe();
    let custom_callback_pipe = create_pipe();

    // Attach the reading end of a pipe to a callback, too
    // 将管道的读取端也附加到回调
    looper
        .as_foreign()
        .add_fd_with_callback(
            unsafe { BorrowedFd::borrow_raw(custom_callback_pipe[0]) },
            FdEvent::INPUT,
            |fd, _| {
                let mut recv = !0u32;
                assert_eq!(
                    unsafe { libc::read(fd.as_raw_fd(), &mut recv as *mut _ as *mut _, U32_SIZE) }
                        as usize,
                    U32_SIZE
                );
                println!("Read custom event from pipe, in callback: {}", recv);
                // Detach this handler by returning `false` once the count reaches 5
                // 一旦计数达到 5，通过返回“false”来分离此处理程序
                recv < 5
            },
        )
        .expect("Failed to add file descriptor to Looper");

    spawn(move || {
        // Send a "custom event" to the looper every second
        // 每秒向 looper 发送一个“自定义事件”
        for i in 0.. {
            let i_addr = &i as *const _ as *const _;
            sleep(Duration::from_secs(1));
            assert_eq!(
                unsafe { libc::write(custom_event_pipe[1], i_addr, U32_SIZE) },
                U32_SIZE as isize
            );
            assert_eq!(
                unsafe { libc::write(custom_callback_pipe[1], i_addr, U32_SIZE,) },
                U32_SIZE as isize
            );
        }
    });

    let mut exit = false;
    let mut redraw_pending = true;
    let mut render_state: Option<()> = Default::default();

    while !exit {
        app.poll_events(Some(Duration::from_secs(1)) /* timeout */, |event| {
            match event {
                PollEvent::Wake => {
                    info!("Early wake up");
                }
                PollEvent::Timeout => {
                    info!("Timed out");
                    // Real app would probably rely on vblank sync via graphics API...
                    // 真正的应用程序可能会依赖通过图形 API 实现的 vblank 同步...
                    redraw_pending = true;
                }
                PollEvent::Main(main_event) => {
                    info!("Main event: {:?}", main_event);
                    match main_event {
                        MainEvent::SaveState { saver, .. } => {
                            saver.store("foo://bar".as_bytes());
                        }
                        MainEvent::Pause => {}
                        MainEvent::Resume { loader, .. } => {
                            if let Some(state) = loader.load() {
                                if let Ok(uri) = String::from_utf8(state) {
                                    info!("Resumed with saved state = {uri:#?}");
                                }
                            }
                        }
                        MainEvent::InitWindow { .. } => {
                            render_state = Some(());
                            redraw_pending = true;
                        }
                        MainEvent::TerminateWindow { .. } => {
                            render_state = None;
                        }
                        MainEvent::WindowResized { .. } => {
                            redraw_pending = true;
                        }
                        MainEvent::RedrawNeeded { .. } => {
                            redraw_pending = true;
                        }
                        MainEvent::InputAvailable { .. } => {
                            redraw_pending = true;
                        }
                        MainEvent::ConfigChanged { .. } => {
                            info!("Config Changed: {:#?}", app.config());
                        }
                        MainEvent::LowMemory => {}

                        MainEvent::Destroy => exit = true,
                        _ => { /* ... */ }
                    }
                }
                _ => {}
            }

            if redraw_pending {
                if let Some(_rs) = render_state {
                    redraw_pending = false;

                    // Handle input
                    // 处理输入
                    match app.input_events_iter() {
                        Ok(mut it) => loop {
                            it.next(|event| {
                                info!("Input Event: {event:?}");
                                InputStatus::Unhandled
                            });
                        },
                        Err(e) => error!("{}", e),
                    }

                    info!("Render...");
                }
            }
        });
    }
}
