#![allow(dead_code)]

use rstest::fixture;
use std::{
    net::SocketAddr,
    sync::{Arc, OnceLock},
    thread::JoinHandle,
};
use thirtyfour::prelude::*;
use thirtyfour::support::block_on;
use thirtyfour_testing_library_ext::Screen;
use tokio::sync::{Semaphore, SemaphorePermit};

static SERVER: OnceLock<Arc<JoinHandle<()>>> = OnceLock::new();
static LOGINIT: OnceLock<()> = OnceLock::new();

const ASSETS_DIR: &str = "tests/test_html";
const PORT: u16 = 8081;

/// Create the Capabilities struct for the specified browser.
pub fn make_capabilities(s: &str) -> Capabilities {
    match s {
        "firefox" => {
            let mut caps = DesiredCapabilities::firefox();
            caps.set_headless().unwrap();
            caps.into()
        }
        "chrome" => {
            let mut caps = DesiredCapabilities::chrome();
            caps.set_headless().unwrap();
            caps.set_no_sandbox().unwrap();
            caps.set_disable_gpu().unwrap();
            caps.set_disable_dev_shm_usage().unwrap();
            caps.add_arg("--no-sandbox").unwrap();
            caps.into()
        }
        browser => unimplemented!("unsupported browser backend {}", browser),
    }
}

/// Get the WebDriver URL for the specified browser.
pub fn webdriver_url(s: &str) -> String {
    match s {
        "firefox" => "http://localhost:4444".to_string(),
        "chrome" => "http://localhost:9515".to_string(),
        browser => unimplemented!("unsupported browser backend {}", browser),
    }
}

/// Starts the web server.
pub fn start_server() -> Arc<JoinHandle<()>> {
    SERVER
        .get_or_init(|| {
            let handle = std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                rt.block_on(async {
                    tracing::debug!("starting web server on http://localhost:{PORT}");
                    let addr = SocketAddr::from(([127, 0, 0, 1], PORT));
                    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
                    let app = axum::Router::new()
                        .fallback_service(tower_http::services::ServeDir::new(ASSETS_DIR));
                    axum::serve(listener, app).await.unwrap();
                });
            });
            Arc::new(handle)
        })
        .clone()
}

pub fn init_logging() {
    LOGINIT.get_or_init(|| {
        use tracing_subscriber::{fmt, prelude::*, EnvFilter};
        tracing_subscriber::registry()
            .with(fmt::layer())
            .with(EnvFilter::from_default_env())
            .init();
    });
}

/// Get the global limiter mutex.
pub fn get_limiter() -> &'static Semaphore {
    static LIMITER: Semaphore = Semaphore::const_new(1);

    &LIMITER
}

/// Locks the Firefox browser for exclusive use.
///
/// This ensures there is only ever one Firefox browser running at a time.
pub async fn lock_firefox<'a>(browser: &str) -> Option<SemaphorePermit<'static>> {
    if browser == "firefox" {
        Some(get_limiter().acquire().await.unwrap())
    } else {
        None
    }
}

/// Launch the specified browser.
pub async fn launch_browser(browser: &str) -> WebDriver {
    tracing::debug!("launching browser {browser}");
    let caps = make_capabilities(browser);
    let webdriver_url = webdriver_url(browser);
    WebDriver::new(webdriver_url, caps)
        .await
        .expect("Failed to create WebDriver")
}

/// Helper struct for running tests.
pub struct TestHarness {
    browser: String,
    server: Arc<JoinHandle<()>>,
    driver: Option<WebDriver>,
    guard: Option<SemaphorePermit<'static>>,
}

impl TestHarness {
    /// Create a new TestHarness instance.
    pub async fn new(browser: &str) -> Self {
        init_logging();
        let server = start_server();
        let guard = lock_firefox(browser).await;
        let driver = Some(launch_browser(browser).await);
        Self {
            browser: browser.to_string(),
            server,
            driver,
            guard,
        }
    }

    /// Get the browser name.
    pub fn browser(&self) -> &str {
        &self.browser
    }

    /// Get the WebDriver instance.
    pub fn driver(&self) -> &WebDriver {
        self.driver.as_ref().expect("the driver to still be active")
    }

    /// Navigate to a test HTML file and get a configured screen.
    pub async fn screen_for_page(&self, html_file: &str) -> WebDriverResult<Screen> {
        let url = format!("http://localhost:{PORT}/{html_file}");
        self.driver().goto(&url).await?;
        Ok(self.screen().await)
    }

    async fn screen(&self) -> Screen {
        Screen::build_with_testing_library(self.driver().clone())
            .await
            .expect("Failed to create Screen")
    }

    /// Disable auto-closing the browser when the TestHarness is dropped.
    pub fn disable_auto_close(mut self) -> Self {
        if let Some(driver) = self.driver.take() {
            let _ = driver.leak();
        }
        self
    }
}

/// Fixture for running tests.
#[fixture]
pub fn test_harness() -> TestHarness {
    let browser = std::env::var("THIRTYFOUR_BROWSER").unwrap_or_else(|_| "chrome".to_string());
    block_on(TestHarness::new(&browser))
}

pub fn sample_page_url() -> String {
    format!("http://localhost:{PORT}/sample_page.html")
}

pub fn other_page_url() -> String {
    format!("http://localhost:{PORT}/other_page.html")
}

pub fn drag_to_url() -> String {
    format!("http://localhost:{PORT}/drag_to.html")
}

pub fn by_role_options_page_url() -> String {
    format!("http://localhost:{PORT}/by_role_options.html")
}

pub fn by_label_text_options_page_url() -> String {
    format!("http://localhost:{PORT}/by_label_text_options.html")
}

pub fn by_text_exact_page_url() -> String {
    format!("http://localhost:{PORT}/by_text_exact.html")
}

pub fn by_placeholder_text_exact_page_url() -> String {
    format!("http://localhost:{PORT}/by_placeholder_text_exact.html")
}

pub fn by_display_value_exact_page_url() -> String {
    format!("http://localhost:{PORT}/by_display_value_exact.html")
}

pub fn by_alt_text_exact_page_url() -> String {
    format!("http://localhost:{PORT}/by_alt_text_exact.html")
}

pub fn by_title_exact_page_url() -> String {
    format!("http://localhost:{PORT}/by_title_exact.html")
}

pub fn by_test_id_exact_page_url() -> String {
    format!("http://localhost:{PORT}/by_test_id_exact.html")
}

pub fn screen_within_page_url() -> String {
    format!("http://localhost:{PORT}/screen_within.html")
}

pub fn screen_configure_page_url() -> String {
    format!("http://localhost:{PORT}/screen_configure.html")
}

pub async fn assert_id(element: &WebElement, expected: &str) -> WebDriverResult<()> {
    let actual_id = element.id().await?.unwrap();
    assert_eq!(actual_id, expected);
    Ok(())
}

pub async fn assert_text(element: &WebElement, expected: &str) -> WebDriverResult<()> {
    let actual_text = element.text().await?;
    assert_eq!(actual_text, expected);
    Ok(())
}

pub fn assert_none<T>(result: Option<T>) -> WebDriverResult<()> {
    assert!(result.is_none());
    Ok(())
}

pub fn assert_error<T>(result: WebDriverResult<T>) -> WebDriverResult<()> {
    assert!(result.is_err());
    Ok(())
}

pub fn assert_count<T>(elements: &[T], expected: usize) -> WebDriverResult<()> {
    assert_eq!(elements.len(), expected);
    Ok(())
}
