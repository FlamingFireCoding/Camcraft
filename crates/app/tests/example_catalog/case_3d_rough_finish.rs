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

    /// #3670: the 3D Pocket's entry is a helix. A pass that restarts in a
    /// level after a hop over the dome must not feed straight down into
    /// uncut stock: it takes the entry type, or it goes down only where
    /// the column is already cleared (the centre-cutting plunge law).
    #[test]
    fn pocket_re_entries_never_plunge_into_uncut_stock() {
        use camcraft_app::{model, operation, project, setup, world};
        use camcraft_motion::plunge_law::{Solid, first_solid_plunge};
        let dir = repo("examples/catalog/3d/rough-finish");
        let w = world::new_world();
        let notes = project::load_noted(&w, &dir.join("rough-finish.camproj")).expect("rough-finish project");
        assert!(notes.is_empty(), "load notes: {notes:?}");
        let (_, pocket) = setup::operation_order(&w)
            .into_iter()
            .find(|(n, _)| n == "3D Pocket1")
            .expect("the 3D Pocket of the case");
        let op = model::operation_of(&w, &pocket).expect("pocket operation");
        let (t, _) = operation::generate(&w, &op, "3D Pocket1").expect("the pocket generates");
        // Stock box: 22 mm tall, its top at Z22 (clearance 47 = top + 25).
        let solid = Solid { top: 22.0, footprint: None };
        assert_eq!(first_solid_plunge(&t, 3.0, &solid), None, "a re-entry plunges into uncut stock: {:?}", t.notes);
    }
}
