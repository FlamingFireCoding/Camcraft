// Provenance: original CamCraft 3D rough-and-finish catalog contract (#2866).
mod catalog_3d_rough_finish {
    use super::*;

    #[test]
    fn regenerates() {
        run_case("3d", "rough-finish", "regenerates", regenerate);
    }

    #[test]
    fn rebuilds() {
        run_case("3d", "rough-finish", "rebuilds", rebuild);
    }

    #[test]
    fn interlevel_rapid_stops_at_feed_height_above_prior_floor() {
        // #3671: between levels the tool must not rapid down to the floor
        // of the cut level above with 0 margin. The rapid must stop at feed
        // height (plunge_height = 2.0 mm) above the last cut floor.
        let dir = repo("examples/catalog/3d/rough-finish");
        let nc = std::fs::read_to_string(dir.join("rough-finish.nc")).expect("read nc");
        let op_lines = section(&nc, "3D Pocket1");
        assert!(!op_lines.is_empty(), "3D Pocket1 section present");
        let mut lowest_rapid_z = f64::INFINITY;
        for line in &op_lines {
            if line.starts_with("G00") || line.starts_with("G0 ") {
                if let Some(z) = word(line, 'Z') {
                    if z < lowest_rapid_z {
                        lowest_rapid_z = z;
                    }
                }
            }
        }
        // The last level cuts at Z8. The level above it cut at Z10.
        // The rapid before the last level must stop at Z10 + 2.0 = Z12.
        assert!(
            lowest_rapid_z >= 12.0 - 1e-6,
            "lowest rapid in 3D Pocket1 was Z {lowest_rapid_z}, expected >= 12.0 (feed height above Z10)"
        );
    }
}
