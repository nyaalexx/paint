use std::mem::ManuallyDrop;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread::JoinHandle;

use chrono::Utc;
use paint_core::behaviour::{self, DownloadedTexture, Impls};
use paint_core::persistence::{self, Error, NewProject, Project, Transaction};

#[derive(Debug, thiserror::Error)]
#[error("persistence thread is dead")]
struct PersistenceThreadDeadError;

#[derive(Debug, thiserror::Error)]
#[error("no project open")]
struct NoProjectOpenError;

#[derive(Debug, thiserror::Error)]
#[error("project id mismatch")]
struct ProjectIdMismatch;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ProjectId(u64);

pub struct PersistenceManager<I: Impls> {
    thread: ManuallyDrop<JoinHandle<()>>,
    command_sender: ManuallyDrop<mpsc::Sender<Command<I>>>,
}

impl<I: Impls> PersistenceManager<I> {
    pub fn new() -> Self {
        let (command_sender, command_receiver) = mpsc::channel();

        let join_handle = std::thread::spawn(move || {
            let thread = PersistenceThread::<I> {
                command_receiver,
                project: None,
                project_id: None,
                next_project_id: ProjectId(1),
            };

            thread.run();
        });

        Self {
            thread: ManuallyDrop::new(join_handle),
            command_sender: ManuallyDrop::new(command_sender),
        }
    }

    fn send_command(&self, cmd: Command<I>) -> Result<(), Error> {
        self.command_sender
            .send(cmd)
            .map_err(|_| Error::Internal(PersistenceThreadDeadError.into()))
    }

    pub async fn create_project(&self, new_project: NewProject) -> Result<ProjectId, Error> {
        let (result_sender, result_receiver) = oneshot::channel();

        self.send_command(Command::CreateProject {
            new_project,
            result_sender,
        })?;

        result_receiver
            .await
            .map_err(|_| Error::Internal(PersistenceThreadDeadError.into()))?
    }

    pub async fn open_project(&self, path: impl Into<PathBuf>) -> Result<OpenedProject, Error> {
        let (result_sender, result_receiver) = oneshot::channel();

        self.send_command(Command::OpenProject {
            path: path.into(),
            result_sender,
        })?;

        result_receiver
            .await
            .map_err(|_| Error::Internal(PersistenceThreadDeadError.into()))?
    }

    pub async fn save_project(
        &self,
        id: ProjectId,
        diff: Diff<I>,
        path: impl Into<PathBuf>,
    ) -> Result<(), Error> {
        let (result_sender, result_receiver) = oneshot::channel();

        self.send_command(Command::SaveProject {
            id,
            diff,
            path: path.into(),
            result_sender,
        })?;

        result_receiver
            .await
            .map_err(|_| Error::Internal(PersistenceThreadDeadError.into()))?
    }
}

impl<I: Impls> Drop for PersistenceManager<I> {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.command_sender);
            ManuallyDrop::take(&mut self.thread)
                .join()
                .expect("persistence thread panicked");
        }
    }
}

enum Command<I: Impls> {
    CreateProject {
        new_project: NewProject,
        result_sender: oneshot::Sender<Result<ProjectId, Error>>,
    },
    OpenProject {
        path: PathBuf,
        result_sender: oneshot::Sender<Result<OpenedProject, Error>>,
    },
    SaveProject {
        id: ProjectId,
        diff: Diff<I>,
        path: PathBuf,
        result_sender: oneshot::Sender<Result<(), Error>>,
    },
}

struct PersistenceThread<I: Impls> {
    command_receiver: mpsc::Receiver<Command<I>>,
    // TODO: store multiple projects? idk
    project: Option<I::Project>,
    project_id: Option<ProjectId>,
    next_project_id: ProjectId,
}

impl<I: Impls> PersistenceThread<I> {
    pub fn run(mut self) {
        while let Ok(cmd) = self.command_receiver.recv() {
            match cmd {
                Command::CreateProject {
                    new_project,
                    result_sender,
                } => {
                    let res = self.create_project(new_project);
                    let _ = result_sender.send(res);
                }

                Command::OpenProject {
                    path,
                    result_sender,
                } => {
                    let res = self.open_project(&path);
                    let _ = result_sender.send(res);
                }

                Command::SaveProject {
                    id,
                    diff,
                    path,
                    result_sender,
                } => {
                    let res = self.save_project(id, diff, &path);
                    let _ = result_sender.send(res);
                }
            };
        }
    }

    fn create_project(&mut self, new_project: NewProject) -> Result<ProjectId, Error> {
        let project = I::Project::create(new_project)?;
        self.project = Some(project);

        let id = self.next_project_id;
        self.project_id = Some(id);
        self.next_project_id.0 += 1;

        Ok(id)
    }

    fn open_project(&mut self, path: &Path) -> Result<OpenedProject, Error> {
        let project = I::Project::open(path)?;
        let texture = project.get_texture()?;
        self.project = Some(project);

        let id = self.next_project_id;
        self.project_id = Some(id);
        self.next_project_id.0 += 1;

        Ok(OpenedProject { id, texture })
    }

    fn save_project(&mut self, id: ProjectId, diff: Diff<I>, path: &Path) -> Result<(), Error> {
        if Some(id) != self.project_id {
            return Err(Error::Internal(ProjectIdMismatch.into()));
        };

        let Some(project) = &mut self.project else {
            return Err(Error::Internal(NoProjectOpenError.into()));
        };

        if project.path() != Some(path) {
            *project = project.save_copy(path)?;
        }

        let mut tx = project.begin_change()?;

        tx.set_texture(&diff.new_texture.as_persistence())?;

        tx.update_metadata(|meta| {
            meta.change_time = Utc::now();
        })?;

        tx.commit()?;

        Ok(())
    }
}

pub struct Diff<I: Impls> {
    pub new_texture: <I::Texture as behaviour::Texture>::Downloaded,
}

pub struct OpenedProject {
    pub id: ProjectId,
    pub texture: persistence::Texture<'static>,
}
