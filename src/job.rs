use failure::Error;
use serde::Deserializer;

use Jenkins;
use action::Action;
use build::ShortBuild;
use client::{self, Name, Path};
use job_builder::JobBuilder;
use queue::ShortQueueItem;

/// Ball Color corresponding to a `BuildStatus`
#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum BallColor {
    /// Success
    Blue,
    /// Success, and build is on-going
    BlueAnime,
    /// Unstable
    Yellow,
    /// Unstable, and build is on-going
    YellowAnime,
    /// Failure
    Red,
    /// Failure, and build is on-going
    RedAnime,
    /// Catch-all for disabled, aborted, not yet build
    Grey,
    /// Catch-all for disabled, aborted, not yet build, and build is on-going
    GreyAnime,
    /// Disabled
    Disabled,
    /// Disabled, and build is on-going
    DisabledAnime,
    /// Aborted
    Aborted,
    ///Aborted, and build is on-going
    AbortedAnime,
    /// Not Build
    #[serde(rename = "notbuilt")]
    NotBuilt,
    /// Not Build, and build is on-going
    #[serde(rename = "notbuilt_anime")]
    NotBuiltAnime,
}
impl Default for BallColor {
    fn default() -> Self {
        BallColor::NotBuilt
    }
}

/// Short Job that is used in lists and links from other structs
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ShortJob {
    /// Name of the job
    pub name: String,
    /// URL for the job
    pub url: String,
    /// Ball Color for the status of the job
    pub color: BallColor,
}
impl ShortJob {
    /// Get the full details of a `Job` matching the `ShortJob`
    pub fn get_full_job(&self, jenkins_client: &Jenkins) -> Result<Job, Error> {
        let path = jenkins_client.url_to_path(&self.url);
        if let Path::Job { .. } = path {
            Ok(jenkins_client.get(&path)?.json()?)
        } else {
            Err(client::Error::InvalidUrl {
                url: self.url.clone(),
                expected: client::error::ExpectedType::Job,
            }.into())
        }
    }
}

tagged_enum_or_default!(
    /// A Jenkins `Job`
    pub enum Job {
        common_fields {
            /// Name of the job
            name: String,
            /// Display Name of the job
            display_name: String,
            /// Full Display Name of the job
            full_display_name: String,
            /// Full Name of the job
            full_name: String,
            /// Display Name of the job
            display_name_or_null: Option<String>,
            /// Description of the job
            description: String,
            /// URL for the job
            url: String,
            /// Ball Color for the status of the job
            color: BallColor,
            /// Is the job buildable?
            buildable: bool,
            /// Are dependencies kept for this job?
            keep_dependencies: bool,
            /// Next build number
            next_build_number: u32,
            /// Is this job currently in build queue
            in_queue: bool,
            /// Actions of a job
            actions: Vec<Option<Action>>,
            /// Link to the last build
            last_build: Option<ShortBuild>,
            /// Link to the first build
            first_build: Option<ShortBuild>,
            /// Link to the last stable build
            last_stable_build: Option<ShortBuild>,
            /// Link to the last unstable build
            last_unstable_build: Option<ShortBuild>,
            /// Link to the last successful build
            last_successful_build: Option<ShortBuild>,
            /// Link to the last unsucressful build
            last_unsuccessful_build: Option<ShortBuild>,
            /// Link to the last complete build
            last_completed_build: Option<ShortBuild>,
            /// Link to the last failed build
            last_failed_build: Option<ShortBuild>,
            /// List of builds of the job
            builds: Vec<ShortBuild>,
            /// HealthReport of the job
            health_report: Vec<HealthReport>,
            /// Queue item of this job if it's waiting
            queue_item: Option<ShortQueueItem>,
            /// Properties of the job
            property: Vec<Property>
        };
        /// A free style project
        FreeStyleProject (_class = "hudson.model.FreeStyleProject") {
            /// Is concurrent build enabled for the job?
            concurrent_build: bool,
            /// SCM configured for the job
            scm: SCM,
            /// List of the upstream projects
            upstream_projects: Vec<ShortJob>,
            /// List of the downstream projects
            downstream_projects: Vec<ShortJob>,
            /// Label expression
            label_expression: Option<String>
        },
        /// A pipeline project
        WorkflowJob (_class = "org.jenkinsci.plugins.workflow.job.WorkflowJob") {
            /// Is concurrent build enabled for the job?
            concurrent_build: bool,
        },
        /// A matrix project
        MatrixProject (_class = "hudson.matrix.MatrixProject") {
            /// Is concurrent build enabled for the job?
            concurrent_build: bool,
            /// SCM configured for the job
            scm: SCM,
            /// Configurations for the job
            active_configurations: Vec<ShortJob>,
            /// List of the upstream projects
            upstream_projects: Vec<ShortJob>,
            /// List of the downstream projects
            downstream_projects: Vec<ShortJob>,
            /// Label expression
            label_expression: Option<String>
        },
        /// A matrix configuration
        MatrixConfiguration (_class = "hudson.matrix.MatrixConfiguration") {
            /// Is concurrent build enabled for the job?
            concurrent_build: bool,
            /// SCM configured for the job
            scm: SCM,
            /// List of the upstream projects
            upstream_projects: Vec<ShortJob>,
            /// List of the downstream projects
            downstream_projects: Vec<ShortJob>,
            /// Label expression
            label_expression: Option<String>
        },
        /// An external job
        ExternalJob (_class = "hudson.model.ExternalJob") {
        },
        /// A maven project
        MavenModuleSet (_class = "hudson.maven.MavenModuleSet") {
            /// Is concurrent build enabled for the job?
            concurrent_build: bool,
            /// SCM configured for the job
            scm: SCM,
            /// List of modules
            modules: Vec<ShortJob>,
            /// List of the upstream projects
            upstream_projects: Vec<ShortJob>,
            /// List of the downstream projects
            downstream_projects: Vec<ShortJob>,
            /// Label expression
            label_expression: Option<String>
        },
        /// A maven module
        MavenModule (_class = "hudson.maven.MavenModule") {
            /// Is concurrent build enabled for the job?
            concurrent_build: bool,
            /// SCM configured for the job
            scm: SCM,
            /// List of the upstream projects
            upstream_projects: Vec<ShortJob>,
            /// List of the downstream projects
            downstream_projects: Vec<ShortJob>,
            /// Label expression
            label_expression: Option<String>
        }
    }
);

macro_rules! job_common_fields_dispatch {
    ($field:ident -> $return:ty) => {
        pub(crate) fn $field(&self) -> Result<$return, Error> {
            match self {
                &Job::FreeStyleProject { ref $field, .. } => Ok($field),
                &Job::WorkflowJob { ref $field, .. } => Ok($field),
                &Job::MatrixProject { ref $field, .. } => Ok($field),
                &Job::MatrixConfiguration { ref $field, .. } => Ok($field),
                &Job::ExternalJob { ref $field, .. } => Ok($field),
                &Job::MavenModuleSet { ref $field, .. } => Ok($field),
                &Job::MavenModule { ref $field, .. } => Ok($field),
                x @ &Job::Unknown { .. } => Err(client::Error::InvalidObjectType {
                    object_type: client::error::ExpectedType::Job,
                    action: client::error::Action::GetField(stringify!($field)),
                    variant_name: x.variant_name().to_string(),
                }.into()),
            }
        }
    };
    ($(#[$attr:meta])* pub $field:ident -> $return:ty) => {
        $(#[$attr])*
        pub fn $field(&self) -> Result<$return, Error> {
            match self {
                &Job::FreeStyleProject { $field, .. } => Ok($field),
                &Job::WorkflowJob { $field, .. } => Ok($field),
                &Job::MatrixProject { $field, .. } => Ok($field),
                &Job::MatrixConfiguration { $field, .. } => Ok($field),
                &Job::ExternalJob { $field, .. } => Ok($field),
                &Job::MavenModuleSet { $field, .. } => Ok($field),
                &Job::MavenModule { $field, .. } => Ok($field),
                x @ &Job::Unknown { .. } => Err(client::Error::InvalidObjectType {
                    object_type: client::error::ExpectedType::Job,
                    action: client::error::Action::GetField(stringify!($field)),
                    variant_name: x.variant_name().to_string(),
                }.into()),
            }
        }
    };
    ($(#[$attr:meta])* pub ref $field:ident -> $return:ty) => {
        $(#[$attr])*
        pub fn $field(&self) -> Result<$return, Error> {
            match self {
                &Job::FreeStyleProject { ref $field, .. } => Ok($field),
                &Job::WorkflowJob { ref $field, .. } => Ok($field),
                &Job::MatrixProject { ref $field, .. } => Ok($field),
                &Job::MatrixConfiguration { ref $field, .. } => Ok($field),
                &Job::ExternalJob { ref $field, .. } => Ok($field),
                &Job::MavenModuleSet { ref $field, .. } => Ok($field),
                &Job::MavenModule { ref $field, .. } => Ok($field),
                x @ &Job::Unknown { .. } => Err(client::Error::InvalidObjectType {
                    object_type: client::error::ExpectedType::Job,
                    action: client::error::Action::GetField(stringify!($field)),
                    variant_name: x.variant_name().to_string(),
                }.into()),
            }
        }
    };
}

impl Job {
    job_common_fields_dispatch!(url -> &str);
    job_common_fields_dispatch!(
        /// Get the name of the project
        pub ref name -> &str
    );
    job_common_fields_dispatch!(
        /// Is the project buildable
        pub buildable -> bool
    );
    job_common_fields_dispatch!(
        /// Link to the last build
        pub ref last_build -> &Option<ShortBuild>
    );
    job_common_fields_dispatch!(
        /// List of builds of the job
        pub ref builds -> &Vec<ShortBuild>
    );
    job_common_fields_dispatch!(
        /// Health report of the project
        pub ref health_report -> &Vec<HealthReport>
    );

    /// Enable a `Job`. It may need to be refreshed as it may have been updated
    pub fn enable(&self, jenkins_client: &Jenkins) -> Result<(), Error> {
        let path = jenkins_client.url_to_path(&self.url()?);
        if let Path::Job {
            name,
            configuration: None,
        } = path
        {
            jenkins_client.post(&Path::JobEnable { name })?;
            Ok(())
        } else {
            Err(client::Error::InvalidUrl {
                url: self.url()?.to_string(),
                expected: client::error::ExpectedType::Job,
            }.into())
        }
    }

    /// Disable a `Job`. It may need to be refreshed as it may have been updated
    pub fn disable(&self, jenkins_client: &Jenkins) -> Result<(), Error> {
        let path = jenkins_client.url_to_path(&self.url()?);
        if let Path::Job {
            name,
            configuration: None,
        } = path
        {
            jenkins_client.post(&Path::JobDisable { name })?;
            Ok(())
        } else {
            Err(client::Error::InvalidUrl {
                url: self.url()?.to_string(),
                expected: client::error::ExpectedType::Job,
            }.into())
        }
    }

    /// Add this job to the view `view_name`
    pub fn add_to_view(&self, jenkins_client: &Jenkins, view_name: &str) -> Result<(), Error> {
        let path = jenkins_client.url_to_path(&self.url()?);
        if let Path::Job {
            name,
            configuration: None,
        } = path
        {
            jenkins_client.post(&Path::AddJobToView {
                job_name: name,
                view_name: Name::Name(view_name),
            })?;
            Ok(())
        } else {
            Err(client::Error::InvalidUrl {
                url: self.url()?.to_string(),
                expected: client::error::ExpectedType::Job,
            }.into())
        }
    }

    /// Remove this job from the view `view_name`
    pub fn remove_from_view(&self, jenkins_client: &Jenkins, view_name: &str) -> Result<(), Error> {
        let path = jenkins_client.url_to_path(&self.url()?);
        if let Path::Job {
            name,
            configuration: None,
        } = path
        {
            jenkins_client.post(&Path::RemoveJobFromView {
                job_name: name,
                view_name: Name::Name(view_name),
            })?;
            Ok(())
        } else {
            Err(client::Error::InvalidUrl {
                url: self.url()?.to_string(),
                expected: client::error::ExpectedType::Job,
            }.into())
        }
    }

    /// Build this job
    pub fn build(&self, jenkins_client: &Jenkins) -> Result<ShortQueueItem, Error> {
        self.builder(jenkins_client)?.send()
    }

    /// Create a `JobBuilder` to setup a build of a `Job`
    pub fn builder<'a, 'b, 'c, 'd>(
        &'a self,
        jenkins_client: &'b Jenkins,
    ) -> Result<JobBuilder<'a, 'b, 'c, 'd>, Error> {
        JobBuilder::new(self, jenkins_client)
    }

    /// Poll configured SCM for changes
    pub fn poll_scm(&self, jenkins_client: &Jenkins) -> Result<(), Error> {
        let path = jenkins_client.url_to_path(&self.url()?);
        if let Path::Job {
            name,
            configuration: None,
        } = path
        {
            jenkins_client.post(&Path::PollSCMJob { name })?;
            Ok(())
        } else {
            Err(client::Error::InvalidUrl {
                url: self.url()?.to_string(),
                expected: client::error::ExpectedType::Job,
            }.into())
        }
    }
}

impl Jenkins {
    /// Get a `Job` from it's `job_name`
    pub fn get_job(&self, job_name: &str) -> Result<Job, Error> {
        Ok(self.get(&Path::Job {
            name: Name::Name(job_name),
            configuration: None,
        })?
            .json()?)
    }

    /// Build a `Job` from it's `job_name`
    pub fn build_job(&self, job_name: &str) -> Result<ShortQueueItem, Error> {
        JobBuilder::new_from_job_name(job_name, self)?.send()
    }

    /// Create a `JobBuilder` to setup a build of a `Job` from it's `job_name`
    pub fn job_builder<'a, 'b, 'c, 'd>(
        &'b self,
        job_name: &'a str,
    ) -> Result<JobBuilder<'a, 'b, 'c, 'd>, Error> {
        JobBuilder::new_from_job_name(job_name, self)
    }

    /// Poll SCM of a `Job` from it's `job_name`
    pub fn poll_scm_job(&self, job_name: &str) -> Result<(), Error> {
        self.post(&Path::PollSCMJob {
            name: Name::Name(job_name),
        })?;
        Ok(())
    }
}

/// Health Report of a `Job`
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HealthReport {
    /// Description of the `HealthReport`
    pub description: String,
    /// Icon name
    pub icon_class_name: String,
    /// Icon url
    pub icon_url: String,
    /// Score of the `Job`
    pub score: u16,
}

tagged_enum_or_default!(
    /// An SCM
    pub enum SCM {
        /// No SCM
        NullSCM (_class = "hudson.scm.NullSCM") {
            /// Browser
            browser: Option<Browser>,
        },
        /// Git SCM
        GitSCM (_class = "hudson.plugins.git.GitSCM") {
            /// Browser
            browser: Option<Browser>,
            /// Merge options
            merge_options: MergeOptions,
        },
    }
);

impl Default for SCM {
    fn default() -> Self {
        SCM::NullSCM {
            browser: None,
        }
    }
}

tagged_enum_or_default!(
    /// A property of a job
    pub enum Property {
        /// Job is a GitHub project
        GithubProjectProperty (_class = "com.coravy.hudson.plugins.github.GithubProjectProperty") {},
        /// Job is limited in number of builds
        RateLimitBranchProperty (_class = "jenkins.branch.RateLimitBranchProperty$JobPropertyImpl") {},
        /// Old builds of job are discarded
        BuildDiscarderProperty (_class = "jenkins.model.BuildDiscarderProperty") {},
    }
);

tagged_enum_or_default!(
    /// A browser
    pub enum Browser {
        /// Github web browser
        GithubWeb (_class = "hudson.plugins.git.browser.GithubWeb") {},
    }
);

impl Default for Browser {
    fn default() -> Self {
        Browser::Unknown { class: None }
    }
}

/// SCM merge options
#[derive(Default, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MergeOptions {
    /// Merge strategy
    merge_strategy: String,
    /// Fast forward mode
    fast_forward_mode: String,
    /// Merge target
    merge_target: Option<String>,
    /// Remote branch
    remote_branch_name: Option<String>,
}
