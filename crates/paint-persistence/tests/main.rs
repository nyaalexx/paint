use chrono::{TimeDelta, Utc};
use glam::UVec2;
use paint_core::persistence::{Metadata, Project as _, Result, Transaction as _};
use paint_persistence::Project;
use tempfile::tempdir;

#[test]
fn create_project() -> Result<()> {
    let metadata = Metadata {
        creation_time: Utc::now(),
        change_time: Utc::now(),
        resolution: UVec2::new(128, 128),
    };

    let proj = Project::create(metadata.clone())?;
    assert!(proj.get_metadata()?.approx_eq(&metadata));

    Ok(())
}

#[test]
fn save_project() -> Result<()> {
    let tempdir = tempdir().unwrap();
    let path = tempdir.path().join("project");

    let metadata = Metadata {
        creation_time: Utc::now(),
        change_time: Utc::now(),
        resolution: UVec2::new(128, 128),
    };

    let proj = Project::create(metadata.clone())?;
    proj.save_copy(&path)?;
    drop(proj);

    let proj = Project::open(&path)?;
    assert!(proj.get_metadata()?.approx_eq(&metadata));

    Ok(())
}

#[test]
fn change_project() -> Result<()> {
    let metadata = Metadata {
        creation_time: Utc::now(),
        change_time: Utc::now(),
        resolution: UVec2::new(128, 128),
    };

    let new_metadata = Metadata {
        change_time: metadata.change_time + TimeDelta::seconds(10),
        resolution: UVec2::new(256, 1024),
        ..metadata
    };

    let proj = Project::create(metadata.clone())?;

    let mut tx = proj.begin_change()?;
    tx.update_metadata(|m| {
        assert!(m.approx_eq(&metadata));
        *m = new_metadata.clone()
    })?;
    tx.commit()?;

    assert!(proj.get_metadata()?.approx_eq(&new_metadata));

    Ok(())
}
