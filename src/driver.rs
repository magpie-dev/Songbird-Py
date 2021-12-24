use std::sync::Arc;

use pyo3::prelude::*;
use songbird::driver::{Bitrate, Driver};
use songbird::id::{ChannelId, GuildId, UserId};
use songbird::Config;
use tokio::sync::Mutex;

use crate::config::PyConfig;
use crate::exceptions::{CouldNotConnectToRTPError, UseAsyncConstructorError};
use crate::source::PySource;
use crate::track_handle::PyTrackHandle;

#[pyclass(name = "Driver")]
pub struct PyDriver {
    driver: Arc<Mutex<Driver>>,
}

#[pymethods]
impl PyDriver {
    #[new]
    fn new() -> PyResult<Self> {
        //! This can not create a Driver so it is raises an exception.
        Err(UseAsyncConstructorError::new_err(
            "`await Driver.create()` should be used to construct this class.",
        ))
    }

    #[staticmethod]
    #[args(config = "None")]
    fn create<'p>(py: Python<'p>, config: Option<&PyConfig>) -> PyResult<&'p PyAny> {
        //! Creates a driver for this class.
        //! Drivers must be created in an event loop so it has to be done like this.
        //!
        //! ```python
        //! from songbird import Driver
        //! ...
        //!
        //! driver = await Driver.create()
        //! ```

        let config: Config = match config {
            Some(py_config) => py_config.config.clone(),
            None => Config::default(),
        };

        pyo3_asyncio::tokio::future_into_py(py, async move {
            // Make the config object
            Ok(PyDriver {
                driver: Arc::new(Mutex::new(Driver::new(config))),
            })
        })
    }

    fn connect<'p>(
        &'p self,
        py: Python<'p>,
        token: String,
        endpoint: String,
        session_id: String,
        guild_id: u64,
        channel_id: u64,
        user_id: u64,
    ) -> PyResult<&'p PyAny> {
        //! Connect to a voice channel
        //! Note: url can start with `wss://` or no protocol.
        //!
        //! #Arguments
        //! * `token` - Token recieved from the Discord gateway. This is not your bot token.
        //! * `endpoint` - Endpoint recieved from Discord gateway.
        //! * `session_id` - Session id recieved from Discord gateway.
        //! * `guild_id` - Guild id you want to connct to.
        //! * `channel_id` - Channel id you want to connect to.
        //! * `user_id` - User id of the current user.
        let driver = self.driver.clone();

        let endpoint = endpoint.replace("wss://", "");

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let res = driver
                .lock()
                .await
                .connect(songbird::ConnectionInfo {
                    channel_id: Some(ChannelId::from(channel_id)),
                    endpoint: endpoint,
                    guild_id: GuildId::from(guild_id),
                    session_id: session_id,
                    token: token,
                    user_id: UserId::from(user_id),
                })
                .await;

            match res {
                Err(err) => Err(CouldNotConnectToRTPError::new_err(format!("{:?}", err))),
                Ok(_) => Ok(()),
            }
        })
    }

    fn leave<'p>(&'p self, py: Python<'p>) -> PyResult<&'p PyAny> {
        //! Disables the driver.
        //! This does not update your voice state to remove you from the voice channel.
        let driver = self.driver.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            driver.lock().await.leave();
            Ok(())
        })
    }

    fn mute<'p>(&'p self, py: Python<'p>) -> PyResult<&'p PyAny> {
        //! Mutes the driver.
        let driver = self.driver.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            driver.lock().await.mute(true);
            Ok(())
        })
    }

    fn unmute<'p>(&'p self, py: Python<'p>) -> PyResult<&'p PyAny> {
        //! Unmutes the driver.
        let driver = self.driver.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            driver.lock().await.mute(false);
            Ok(())
        })
    }

    fn is_muted<'p>(&'p self, py: Python<'p>) -> PyResult<&'p PyAny> {
        //! Returns whether the driver is muted.
        let driver = self.driver.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move { Ok(driver.lock().await.is_mute()) })
    }

    fn play_source<'p>(&'p self, py: Python<'p>, source: &'p PySource) -> PyResult<&'p PyAny> {
        //! Plays a Playable object.
        //! Playable are activated when you try to play them. That means all errors are
        //! thrown in this method.
        let driver = self.driver.clone();
        let source = source.source.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let source = source.lock().await.get_input().await;
            if let Err(err) = source {
                return Err(err);
            }

            let track_handle = driver.lock().await.play_source(source.unwrap());
            Ok(PyTrackHandle::from(track_handle))
        })
    }

    fn play_only_source<'p>(
        &'p self,
        py: Python<'p>,
        source: &'p PySource,
    ) -> PyResult<&'p PyAny> {
        //! Same as `play_source` but stops all other sources from playing.
        let driver = self.driver.clone();
        let source = source.source.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let source = source.lock().await.get_input().await;
            if let Err(err) = source {
                return Err(err);
            }

            let track_handle = driver.lock().await.play_only_source(source.unwrap());
            Ok(PyTrackHandle::from(track_handle))
        })
    }

    fn set_bitrate<'p>(&'p self, py: Python<'p>, bitrate: i32) -> PyResult<&'p PyAny> {
        //! Sets the bitrate to a i32
        let driver = self.driver.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            Ok(driver
                .lock()
                .await
                .set_bitrate(Bitrate::BitsPerSecond(bitrate)))
        })
    }

    fn set_bitrate_to_max<'p>(&'p self, py: Python<'p>) -> PyResult<&'p PyAny> {
        //! Sets the bitrate to a Bitrate::Max
        let driver = self.driver.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            Ok(driver.lock().await.set_bitrate(Bitrate::Max))
        })
    }

    fn set_bitrate_to_auto<'p>(&'p self, py: Python<'p>) -> PyResult<&'p PyAny> {
        //! Sets the bitrate to Bitrate::Auto
        let driver = self.driver.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            Ok(driver.lock().await.set_bitrate(Bitrate::Auto))
        })
    }

    fn stop<'p>(&'p self, py: Python<'p>) -> PyResult<&'p PyAny> {
        //! Stops playing audio from all sources.
        let driver = self.driver.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move { Ok(driver.lock().await.stop()) })
    }

    fn set_config<'p>(&'p self, py: Python<'p>, config: &PyConfig) -> PyResult<&'p PyAny> {
        //! Set the config for this Driver
        let driver = self.driver.clone();
        let config = config.config.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            Ok(driver.lock().await.set_config(config))
        })
    }
}
