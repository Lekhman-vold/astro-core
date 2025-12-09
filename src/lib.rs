use libc::{c_char, c_int};
use std::{
    ffi::CString,
    sync::{Mutex, OnceLock},
};
use thiserror::Error;

mod ffi {
    use libc::{c_char, c_double, c_int};

    pub const SE_SUN: c_int = 0;
    pub const SE_MOON: c_int = 1;
    pub const SE_ASC: usize = 0;
    pub const SE_GREG_CAL: c_int = 1;
    pub const SEFLG_SWIEPH: c_int = 2;
    pub const AS_MAXCH: usize = 256;

    extern "C" {
        pub fn swe_set_ephe_path(path: *const c_char);

        pub fn swe_utc_to_jd(
            year: c_int,
            month: c_int,
            day: c_int,
            hour: c_int,
            minute: c_int,
            second: c_double,
            gregflag: c_int,
            dret: *mut c_double,
            serr: *mut c_char,
        ) -> c_int;

        pub fn swe_calc_ut(
            tjd_ut: c_double,
            ipl: c_int,
            iflag: c_int,
            xx: *mut c_double,
            serr: *mut c_char,
        ) -> c_int;

        pub fn swe_houses_ex(
            tjd_ut: c_double,
            iflag: c_int,
            geolat: c_double,
            geolon: c_double,
            hsys: c_int,
            cusps: *mut c_double,
            ascmc: *mut c_double,
        ) -> c_int;
    }
}

/// Basic data for birth info in UTC.
#[derive(Debug, Clone)]
pub struct BirthData {
    pub year: i32,
    pub month: i32,
    pub day: i32,
    pub hour: i32,   // 0-23, UTC
    pub minute: i32, // 0-59
    pub second: f64, // 0.0-59.999
    pub lat: f64,    // latitude in degrees (+N, -S)
    pub lon: f64,    // longitude in degrees (+E, -W)
}

/// Core chart with three main indicators.
#[derive(Debug, Clone)]
pub struct CoreChart {
    pub sun_sign: String, // "aries", "taurus", ...
    pub moon_sign: String,
    pub asc_sign: String,
}

#[derive(Debug, Error)]
pub enum AstroError {
    #[error("Swiss Ephemeris error: {0}")]
    EphemerisError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

static EPHE_PATH: OnceLock<Mutex<String>> = OnceLock::new();

/// Override the Swiss Ephemeris data path. Defaults to the current directory when unset.
pub fn set_ephe_path(path: &str) {
    let mut guard = ephe_path_store()
        .lock()
        .expect("ephemeris path mutex poisoned");
    *guard = path.to_string();
    let c_path = CString::new(path).expect("ephemeris path must not contain interior null bytes");
    unsafe {
        ffi::swe_set_ephe_path(c_path.as_ptr());
    }
}

/// Calculate the Sun, Moon, and Ascendant signs for the given birth data.
pub fn calculate_core_chart(birth: &BirthData) -> Result<CoreChart, AstroError> {
    apply_ephe_path()?;
    let tjd_ut = julian_day_ut(birth)?;

    let sun_long = body_longitude(tjd_ut, ffi::SE_SUN)?;
    let moon_long = body_longitude(tjd_ut, ffi::SE_MOON)?;
    let asc_long = ascendant_longitude(tjd_ut, birth.lat, birth.lon)?;

    Ok(CoreChart {
        sun_sign: sign_name_from_longitude(sun_long),
        moon_sign: sign_name_from_longitude(moon_long),
        asc_sign: sign_name_from_longitude(asc_long),
    })
}

fn ephe_path_store() -> &'static Mutex<String> {
    EPHE_PATH.get_or_init(|| Mutex::new(String::new()))
}

fn apply_ephe_path() -> Result<(), AstroError> {
    let guard = ephe_path_store()
        .lock()
        .map_err(|_| AstroError::InvalidInput("ephemeris path lock poisoned".to_string()))?;
    let c_path = CString::new(guard.as_str())
        .map_err(|_| AstroError::InvalidInput("ephemeris path contains null byte".into()))?;
    unsafe {
        ffi::swe_set_ephe_path(c_path.as_ptr());
    }
    Ok(())
}

fn julian_day_ut(birth: &BirthData) -> Result<f64, AstroError> {
    let mut dret = [0f64; 2];
    let mut serr = [0 as c_char; ffi::AS_MAXCH];
    let rc = unsafe {
        ffi::swe_utc_to_jd(
            birth.year as c_int,
            birth.month as c_int,
            birth.day as c_int,
            birth.hour as c_int,
            birth.minute as c_int,
            birth.second,
            ffi::SE_GREG_CAL,
            dret.as_mut_ptr(),
            serr.as_mut_ptr(),
        )
    };
    if rc < 0 {
        return Err(AstroError::EphemerisError(error_string(&serr)));
    }
    // dret[1] = UT
    Ok(dret[1])
}

fn body_longitude(tjd_ut: f64, ipl: c_int) -> Result<f64, AstroError> {
    let mut xx = [0f64; 6];
    let mut serr = [0 as c_char; ffi::AS_MAXCH];
    let rc = unsafe {
        ffi::swe_calc_ut(
            tjd_ut,
            ipl,
            ffi::SEFLG_SWIEPH,
            xx.as_mut_ptr(),
            serr.as_mut_ptr(),
        )
    };
    if rc < 0 {
        return Err(AstroError::EphemerisError(error_string(&serr)));
    }
    Ok(xx[0])
}

fn ascendant_longitude(tjd_ut: f64, lat: f64, lon: f64) -> Result<f64, AstroError> {
    let mut cusps = [0f64; 13];
    let mut ascmc = [0f64; 10];
    let rc = unsafe {
        ffi::swe_houses_ex(
            tjd_ut,
            ffi::SEFLG_SWIEPH,
            lat,
            lon,
            'P' as c_int,
            cusps.as_mut_ptr(),
            ascmc.as_mut_ptr(),
        )
    };
    if rc < 0 {
        return Err(AstroError::EphemerisError(
            "failed to compute ascendant".to_string(),
        ));
    }
    Ok(ascmc[ffi::SE_ASC])
}

fn error_string(buf: &[c_char]) -> String {
    let nul = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    let bytes: Vec<u8> = buf[..nul].iter().map(|&c| c as u8).collect();
    if bytes.is_empty() {
        "unknown Swiss Ephemeris error".to_string()
    } else {
        String::from_utf8_lossy(&bytes).into_owned()
    }
}

const ZODIAC_SIGNS: [&str; 12] = [
    "aries",
    "taurus",
    "gemini",
    "cancer",
    "leo",
    "virgo",
    "libra",
    "scorpio",
    "sagittarius",
    "capricorn",
    "aquarius",
    "pisces",
];

pub fn sign_name_from_longitude(lon: f64) -> String {
    let mut norm = lon % 360.0;
    if norm < 0.0 {
        norm += 360.0;
    }
    let index = (norm / 30.0).floor() as usize % ZODIAC_SIGNS.len();
    ZODIAC_SIGNS[index].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn calculates_core_chart() {
        let ephe_path = "src/swisseph/ephe";
        if !Path::new(ephe_path).exists() {
            eprintln!(
                "skipping calculates_core_chart: missing ephemeris data at {}",
                ephe_path
            );
            return;
        }
        set_ephe_path(ephe_path);
        let birth = BirthData {
            year: 1990,
            month: 1,
            day: 1,
            hour: 0,
            minute: 0,
            second: 0.0,
            lat: 0.0,
            lon: 0.0,
        };

        let chart = calculate_core_chart(&birth).expect("chart should compute");

        assert!(ZODIAC_SIGNS.contains(&chart.sun_sign.as_str()));
        assert!(ZODIAC_SIGNS.contains(&chart.moon_sign.as_str()));
        assert!(ZODIAC_SIGNS.contains(&chart.asc_sign.as_str()));
    }
}
