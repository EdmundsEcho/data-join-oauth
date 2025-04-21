// {
//   "request": "v1/filesystem/readdir?path=filename",
//   "response": {
//     "count": 2,
//     "status": "Successful",
//     "results": [
//       {
//         "created_at": "2021-11-12 11:52:17",
//         "display_name": "target_list.csv",
//         "is_directory": false,
//         "modified_at": "2021-11-12 11:52:17",
//         "path": "/Users/edmund/Desktop/data/target_list/target_list.csv",
//         "size": 2211054
//       },
//       {
//         "created_at": "2021-11-12 11:52:17",
//         "display_name": "target_list_short.csv",
//         "is_directory": false,
//         "modified_at": "2021-11-12 11:52:17",
//         "path": "/Users/edmund/Desktop/data/target_list/target_list_short.csv",
//         "size": 111198
//       }
//     ]
//   }
// }
use crate::models::drive_provider::DriveProvider;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
///
/// Root value for raw data?
///
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RawFiles<RF> {
    #[serde(alias = "files", alias = "value", alias = "entries")]
    inner: Vec<RF>,
}
impl<RF> IntoIterator for RawFiles<RF>
where
    RF: Into<RawFile>,
{
    type Item = RF;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

///
/// Host what is serialized to deliver on the user-agent request
/// for files (endpoint filesystem)
///
#[derive(Default, Debug, Clone, Serialize)]
pub(crate) struct Files {
    kind: Kind,
    path: Option<String>,
    drive_id: Option<String>,
    files: Vec<File>,
}
#[derive(Debug, Clone)]
pub(crate) struct FilesBuilder {
    kind: Kind,
    path: Option<String>,
    drive_id: Option<String>,
    raw_files: RawFiles<RawFile>,
}
///
/// In order to create a builder must have raw data from
/// a data source in order to serialize the seed values. The
/// instantiator is not actually exposed to the public.
///
/// Instead use the constructors.
///
impl FilesBuilder {
    pub fn new(kind: Kind, resp: RawFiles<RawFile>) -> FilesBuilder {
        FilesBuilder {
            kind,
            path: None,
            drive_id: None,
            raw_files: resp,
        }
    }
    pub fn set_path(mut self, path: String) -> Self {
        self.path = Some(path);
        self
    }
    pub fn set_drive_id(mut self, drive_id: String) -> Self {
        self.drive_id = Some(drive_id);
        self
    }
    // convert RawFiles -> Files
    pub fn build(self) -> Files {
        Files {
            kind: self.kind,
            path: self.path,
            drive_id: self.drive_id,
            // Does not need to be generic because uses Vec
            files: self.raw_files.into_iter().map(Into::into).collect(),
        }
    }
}
/* --------------------------------------------------------------------------------------------- */
/* --------------------------------------------------------------------------------------------- */
// Google
/* --------------------------------------------------------------------------------------------- */
///
/// RawFileGoogle -> File
/// implements Deserialize (the intake type)
///
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RawFileGoogle {
    id: String,
    #[serde(rename = "mimeType")]
    mime_type: String,

    // todo chono NaiveDateTime (has Z)
    #[serde(rename = "createdTime")]
    created_time: String,
    #[serde(rename = "modifiedTime")]
    modified_time: String,

    name: String,
    size: Option<String>,
}
impl RawFileGoogle {
    fn is_dir(&self) -> bool {
        self.mime_type.eq("application/vnd.google-apps.folder")
    }
}
impl From<RawFileGoogle> for File {
    fn from(fd: RawFileGoogle) -> File {
        File {
            id: fd.id.clone(),
            name: fd.name.clone(),
            is_directory: fd.is_dir(),
            mime_type: fd.mime_type.clone(),
            created_time: Some(fd.created_time.clone()),
            modified_time: Some(fd.modified_time.clone()),
            size: fd.size,
        }
    }
}
/* --------------------------------------------------------------------------------------------- */
// MSGraph
/* --------------------------------------------------------------------------------------------- */
///
/// RawFileMSGraph -> File
///
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RawFileMSGraph {
    #[serde(rename = "createdDateTime")]
    created_time: String,
    #[serde(rename = "lastModifiedDateTime")]
    modified_time: String,

    id: String,
    name: String,

    #[serde(flatten)]
    file_or_folder: MSGraphFileOrFolder,

    #[allow(dead_code)]
    size: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum MSGraphFileOrFolder {
    File {
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
    Folder {},
}

impl From<RawFileMSGraph> for File {
    fn from(fd: RawFileMSGraph) -> File {
        File {
            id: fd.id.clone(),
            mime_type: match fd.file_or_folder {
                MSGraphFileOrFolder::File { ref mime_type, .. } => mime_type.to_string(),
                _ => "folder".to_string(),
            },
            name: fd.name.clone(),
            is_directory: match fd.file_or_folder {
                MSGraphFileOrFolder::Folder { .. } => true,
                _ => false,
            },
            created_time: Some(fd.created_time.clone()),
            modified_time: Some(fd.modified_time.clone()),
            size: fd.size.map(|v| v.to_string()),
        }
    }
}
/* --------------------------------------------------------------------------------------------- */
// DropBox
/* --------------------------------------------------------------------------------------------- */
///
/// RawFileDropBox -> File
///
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RawFileDropBox {
    id: String,
    #[serde(rename = ".tag")]
    mime_type: String,
    name: String,
    // todo: create time and take the latest of the two
    client_modified: Option<NaiveDateTime>,
    #[allow(dead_code)]
    server_modified: Option<NaiveDateTime>,
    size: Option<String>,
}
impl From<RawFileDropBox> for File {
    fn from(fd: RawFileDropBox) -> File {
        File {
            id: fd.id.clone(),
            name: fd.name.clone(),
            is_directory: fd.mime_type.eq("folder"),
            mime_type: fd.mime_type.clone(),
            created_time: None,
            modified_time: fd.client_modified.map(|v| v.to_string()),
            size: fd.size,
        }
    }
}
/* --------------------------------------------------------------------------------------------- */
///
/// File implements Serialize (the returned/exported type)
///
#[derive(Default, Debug, Clone, Serialize)]
pub struct File {
    pub id: String,
    pub name: String,
    pub is_directory: bool,
    pub mime_type: String,
    // todo convert to int
    pub size: Option<String>,
    pub created_time: Option<String>,
    pub modified_time: Option<String>,
}
/* --------------------------------------------------------------------------------------------- */

///
/// Task:
/// * connect to protected resource
/// * build the query
/// * transform/build
/// * return the list of files
///
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Kind {
    Google,
    MSGraph,
    DropBox,
    Empty,
}
impl Default for Kind {
    fn default() -> Self {
        Kind::Empty
    }
}
impl From<DriveProvider> for Kind {
    fn from(provider: DriveProvider) -> Kind {
        match provider {
            DriveProvider::Google => Kind::Google,
            DriveProvider::MSGraph => Kind::MSGraph,
            DriveProvider::DropBox => Kind::DropBox,
            _ => Kind::Empty,
        }
    }
}

// Interface for consuming raw input
// ... use constructors
#[derive(Debug, Clone, Deserialize)]
pub(crate) enum RawFile {
    Google(RawFileGoogle),
    MSGraph(RawFileMSGraph),
    DropBox(RawFileDropBox),
}
// constructors
pub(crate) fn google(data: RawFiles<RawFileGoogle>) -> FilesBuilder {
    FilesBuilder::new(
        Kind::Google,
        RawFiles {
            inner: data.into_iter().map(RawFile::Google).collect(),
        },
    )
}
pub(crate) fn ms_graph(data: RawFiles<RawFileMSGraph>) -> FilesBuilder {
    FilesBuilder::new(
        Kind::MSGraph,
        RawFiles {
            inner: data.into_iter().map(RawFile::MSGraph).collect(),
        },
    )
}
pub(crate) fn drop_box(data: RawFiles<RawFileDropBox>) -> FilesBuilder {
    FilesBuilder::new(
        Kind::DropBox,
        RawFiles {
            inner: data.into_iter().map(RawFile::DropBox).collect(),
        },
    )
}

impl From<RawFile> for File {
    fn from(raw_file: RawFile) -> File {
        match raw_file {
            RawFile::Google(f) => f.into(),
            RawFile::MSGraph(f) => f.into(),
            RawFile::DropBox(f) => f.into(),
        }
    }
}
/// Unify the RawFile types from different providers
/// under an enum; this means we have a concrete type
/// for each of the raw data inputs.
/// RawFileProvider -> RawFile enum
impl From<RawFileGoogle> for RawFile {
    fn from(rf: RawFileGoogle) -> RawFile {
        RawFile::Google(rf)
    }
}
impl From<RawFileMSGraph> for RawFile {
    fn from(rf: RawFileMSGraph) -> RawFile {
        RawFile::MSGraph(rf)
    }
}
impl From<RawFileDropBox> for RawFile {
    fn from(rf: RawFileDropBox) -> RawFile {
        RawFile::DropBox(rf)
    }
}
