#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use metar_maker_gui::wx_format::*;
    use metar_maker_gui::config::Units;

    #[test]
    fn wind_basic() {
        assert_eq!(format_wind(Some(270.0), Some(3.0/1.94384), None), "27003KT");
        assert_eq!(format_wind(None, Some(4.0/1.94384), None), "VRB04KT");
        assert_eq!(format_wind(Some(90.0), Some(7.0/1.94384), Some(14.0/1.94384)), "09007KTG14");
    }

    #[test]
    fn vis_quarters() {
        assert_eq!(format_visibility(Some(1609.344), Units::Imperial, &[]), "1SM");
        assert_eq!(format_visibility(Some(804.0), Units::Imperial, &[]), "1/2SM");
        assert_eq!(format_visibility(Some(4828.0), Units::Imperial, &[]), "3SM");
    }

    #[test]
    fn altimeter() {
        assert_eq!(format_pressure(Some(1013.0), Units::Metric), "Q1013");
        assert_eq!(format_pressure(Some(1013.0), Units::Imperial), "A2992");
    }

    #[test]
    fn temp_dew_signs() {
        assert_eq!(format_temp_dew(Some(2.4), Some(-3.4), None), "02/M03");
    }
}
