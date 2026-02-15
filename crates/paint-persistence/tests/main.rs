use chrono::Utc;
use glam::UVec2;
use paint_core::persistence::{Metadata, NewProject, Project as _, Result, Transaction as _};
use paint_persistence::Project;
use tempfile::tempdir;

#[test]
fn create_project() -> Result<()> {
    let new_project = NewProject {
        resolution: UVec2::new(128, 128),
    };

    let expected_metadata = Metadata {
        creation_time: Utc::now(),
        change_time: Utc::now(),
        resolution: UVec2::new(128, 128),
    };

    let proj = Project::create(new_project)?;
    assert!(proj.get_metadata()?.approx_eq(&expected_metadata));

    Ok(())
}

#[test]
fn save_project() -> Result<()> {
    let tempdir = tempdir().unwrap();
    let path = tempdir.path().join("project");

    let new_project = NewProject {
        resolution: UVec2::new(128, 128),
    };

    let expected_metadata = Metadata {
        creation_time: Utc::now(),
        change_time: Utc::now(),
        resolution: UVec2::new(128, 128),
    };

    let proj = Project::create(new_project)?;
    proj.save_copy(&path)?;
    drop(proj);

    let proj = Project::open(&path)?;
    assert!(proj.get_metadata()?.approx_eq(&expected_metadata));

    Ok(())
}

#[test]
fn change_project() -> Result<()> {
    let new_project = NewProject {
        resolution: UVec2::new(128, 128),
    };

    let new_resolution = UVec2::new(256, 1024);

    let mut proj = Project::create(new_project)?;

    let mut tx = proj.begin_change()?;
    tx.update_metadata(|m| m.resolution = new_resolution)?;
    tx.commit()?;

    assert_eq!(proj.get_metadata()?.resolution, new_resolution);

    Ok(())
}
