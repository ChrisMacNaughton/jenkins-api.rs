extern crate env_logger;
#[macro_use]
extern crate serde_derive;

extern crate jenkins_api;

use jenkins_api::JenkinsBuilder;
use std::{thread, time};

use std::sync::{Once, ONCE_INIT};

static INIT: Once = ONCE_INIT;

fn setup() {
    INIT.call_once(|| {
        env_logger::init();
    });
}

static JENKINS_URL: &'static str = "http://localhost:8080";

#[test]
fn can_get_jenkins_home() {
    setup();
    let jenkins = JenkinsBuilder::new(JENKINS_URL)
        .with_user("user", Some("password"))
        .build()
        .unwrap();
    assert!(jenkins.get_home().is_ok());
}

#[test]
fn should_be_forbidden() {
    setup();
    let jenkins = JenkinsBuilder::new(JENKINS_URL)
        .with_user("unknown", Some("password"))
        .build()
        .unwrap();
    let response = jenkins.get_home();
    assert!(response.is_err());
    assert_eq!(
        format!("{:?}", response),
        format!(
            "Err(Error {{ kind: ClientError(Unauthorized), url: Some(\"{}/api/json?depth=1\") }})",
            JENKINS_URL
        )
    );
}

#[test]
fn should_be_connection_error() {
    setup();
    let jenkins = JenkinsBuilder::new("http://localhost:1234")
        .build()
        .unwrap();
    let response = jenkins.get_home();
    assert!(response.is_err());
}

#[test]
fn can_get_view() {
    setup();
    let jenkins = JenkinsBuilder::new(JENKINS_URL)
        .with_user("user", Some("password"))
        .build()
        .unwrap();
    assert!(jenkins.get_view("view disabled").is_ok());
}

#[test]
fn should_get_view_not_found() {
    setup();
    let jenkins = JenkinsBuilder::new(JENKINS_URL)
        .with_user("user", Some("password"))
        .build()
        .unwrap();
    let response = jenkins.get_view("zut");
    assert!(response.is_err());
    assert_eq!(
        format!("{:?}", response),
        format!(
            "Err(Error {{ kind: ClientError(NotFound), url: Some(\"{}/view/zut/api/json?depth=1\") }})",
            JENKINS_URL
        )
    );
}

#[test]
fn can_get_job() {
    setup();
    let jenkins = JenkinsBuilder::new(JENKINS_URL)
        .with_user("user", Some("password"))
        .build()
        .unwrap();
    let job = jenkins.get_job("normal job");
    assert!(job.is_ok());
}

#[test]
fn can_get_build() {
    setup();
    let jenkins = JenkinsBuilder::new(JENKINS_URL)
        .with_user("user", Some("password"))
        .build()
        .unwrap();
    assert!(jenkins.get_build("normal job", 1).is_ok());
}

#[test]
fn can_get_jenkins_view_from_home() {
    setup();
    let jenkins = JenkinsBuilder::new(JENKINS_URL)
        .with_user("user", Some("password"))
        .build()
        .unwrap();
    let home = jenkins.get_home();
    assert!(home.is_ok());
    let home_ok = home.unwrap();
    let first_view = home_ok
        .views
        .iter()
        .filter(|view| view.name().unwrap() == "view disabled")
        .nth(0)
        .unwrap();
    let full_view = first_view.get_full_view(&jenkins);
    assert!(full_view.is_ok());
    let full_job = full_view.unwrap().jobs().unwrap()[0].get_full_job(&jenkins);
    assert!(full_job.is_ok());
}

#[test]
fn can_get_build_from_job_and_back() {
    setup();
    let jenkins = JenkinsBuilder::new(JENKINS_URL)
        .with_user("user", Some("password"))
        .build()
        .unwrap();
    let job = jenkins.get_job("normal job");
    assert!(job.is_ok());
    let job_ok = job.unwrap();
    let last_build = job_ok.last_build().unwrap();
    let build = last_build.as_ref().unwrap().get_full_build(&jenkins);
    assert!(build.is_ok());
    let job_back = build.unwrap().get_job(&jenkins);
    assert!(job_back.is_ok());
    assert_eq!(job_back.unwrap().name().unwrap(), job_ok.name().unwrap());
}

#[test]
fn can_disable_job_and_reenable() {
    setup();
    let jenkins = JenkinsBuilder::new(JENKINS_URL)
        .with_user("user", Some("password"))
        .build()
        .unwrap();
    let job = jenkins.get_job("normal job");
    assert!(job.is_ok());
    let job_ok = job.unwrap();
    assert!(job_ok.buildable().unwrap());

    let disabling = job_ok.disable(&jenkins);
    assert!(disabling.is_ok());
    let job_disabled = jenkins.get_job("normal job");
    assert!(job_disabled.is_ok());
    let job_disabled_ok = job_disabled.unwrap();
    assert!(!job_disabled_ok.buildable().unwrap());

    let enabling = job_disabled_ok.enable(&jenkins);
    assert!(enabling.is_ok());
    let job_enabled = jenkins.get_job("normal job");
    assert!(job_enabled.is_ok());
    let job_enabled_ok = job_enabled.unwrap();
    assert!(job_enabled_ok.buildable().unwrap());
}

#[test]
fn can_add_and_remove_job_from_view_through_view() {
    setup();
    let jenkins = JenkinsBuilder::new(JENKINS_URL)
        .with_user("user", Some("password"))
        .build()
        .unwrap();

    let view = jenkins.get_view("test view");
    assert!(view.is_ok());
    let view_ok = view.unwrap();
    assert!(
        view_ok
            .jobs()
            .unwrap()
            .iter()
            .map(|job| &job.name)
            .find(|job_name| *job_name == "normal job")
            .is_none()
    );

    let job = jenkins.get_job("normal job");
    assert!(job.is_ok());
    let job_ok = job.unwrap();

    let adding = view_ok.add_job(&jenkins, &job_ok.name().unwrap());
    assert!(adding.is_ok());

    let view_with = jenkins.get_view("test view");
    assert!(view_with.is_ok());
    assert!(
        view_with
            .unwrap()
            .jobs()
            .unwrap()
            .iter()
            .map(|job| &job.name)
            .find(|job_name| *job_name == "normal job")
            .is_some()
    );

    let removing = view_ok.remove_job(&jenkins, &job_ok.name().unwrap());
    assert!(removing.is_ok());

    let view_without = jenkins.get_view("test view");
    assert!(view_without.is_ok());
    assert!(
        view_without
            .unwrap()
            .jobs()
            .unwrap()
            .iter()
            .map(|job| &job.name)
            .find(|job_name| *job_name == "normal job")
            .is_none()
    );
}

#[test]
fn can_add_and_remove_job_from_view_through_job() {
    setup();
    let jenkins = JenkinsBuilder::new(JENKINS_URL)
        .with_user("user", Some("password"))
        .build()
        .unwrap();

    let view = jenkins.get_view("test view");
    assert!(view.is_ok());
    let view_ok = view.unwrap();
    assert!(
        view_ok
            .jobs()
            .unwrap()
            .iter()
            .map(|job| &job.name)
            .find(|job_name| *job_name == "pipeline job")
            .is_none()
    );

    let job = jenkins.get_job("pipeline job");
    assert!(job.is_ok());
    let job_ok = job.unwrap();

    let adding = job_ok.add_to_view(&jenkins, &view_ok.name().unwrap());
    assert!(adding.is_ok());

    let view_with = jenkins.get_view("test view");
    assert!(view_with.is_ok());
    assert!(
        view_with
            .unwrap()
            .jobs()
            .unwrap()
            .iter()
            .map(|job| &job.name)
            .find(|job_name| *job_name == "pipeline job")
            .is_some()
    );

    let removing = job_ok.remove_from_view(&jenkins, &view_ok.name().unwrap());
    assert!(removing.is_ok());

    let view_without = jenkins.get_view("test view");
    assert!(view_without.is_ok());
    assert!(
        view_without
            .unwrap()
            .jobs()
            .unwrap()
            .iter()
            .map(|job| &job.name)
            .find(|job_name| *job_name == "pipeline job")
            .is_none()
    );
}

#[test]
fn can_get_queue() {
    setup();
    let jenkins = JenkinsBuilder::new(JENKINS_URL)
        .with_user("user", Some("password"))
        .build()
        .unwrap();
    let job = jenkins.get_job("long job");
    assert!(job.is_ok());
    let job_ok = job.unwrap();
    let triggered = job_ok.build(&jenkins);
    assert!(triggered.is_ok());
    let queue = jenkins.get_queue();
    assert!(queue.is_ok());
}

#[test]
fn can_get_queue_item() {
    setup();
    let jenkins = JenkinsBuilder::new(JENKINS_URL)
        .with_user("user", Some("password"))
        .build()
        .unwrap();

    let job = jenkins.get_job("job name");
    assert!(job.is_ok());
    let triggered = job.unwrap().build(&jenkins);
    assert!(triggered.is_ok());

    let triggered_ok = triggered.unwrap();

    let few_seconds = time::Duration::from_secs(2);
    for _ in 0..5 {
        assert!(triggered_ok.get_full_queue_item(&jenkins).is_ok());
        thread::sleep(few_seconds);
    }
}

#[test]
fn can_get_console() {
    setup();
    let jenkins = JenkinsBuilder::new(JENKINS_URL)
        .with_user("user", Some("password"))
        .build()
        .unwrap();

    let job = jenkins.get_job("pipeline job");
    assert!(job.is_ok());

    let job_ok = job.unwrap();
    let last_build = job_ok.last_build().unwrap();
    let build = last_build.as_ref().unwrap().get_full_build(&jenkins);
    assert!(build.is_ok());

    let build_ok = build.unwrap();
    let console = build_ok.get_console(&jenkins);
    assert!(console.is_ok());
}

#[test]
fn can_get_pipeline() {
    setup();
    let jenkins = JenkinsBuilder::new(JENKINS_URL)
        .with_user("user", Some("password"))
        .build()
        .unwrap();

    let job = jenkins.get_job("pipeline job");
    assert!(job.is_ok());

    let build = jenkins.get_build("pipeline job", 1);
    assert!(build.is_ok());
}

#[test]
fn can_build_job_with_delay() {
    setup();
    let jenkins = JenkinsBuilder::new(JENKINS_URL)
        .with_user("user", Some("password"))
        .build()
        .unwrap();

    let triggered = jenkins
        .job_builder("delayed job")
        .unwrap()
        .with_delay(5000)
        .send();
    let triggered_ok = triggered.unwrap();

    let queue = jenkins.get_queue();
    assert!(queue.is_ok());

    thread::sleep(time::Duration::from_secs(2));
    let queue_item = triggered_ok.get_full_queue_item(&jenkins);
    assert!(queue_item.is_ok());
    assert!(queue_item.unwrap().why.is_some());

    thread::sleep(time::Duration::from_secs(10));

    let queue_item = triggered_ok.get_full_queue_item(&jenkins);
    assert!(queue_item.is_ok());
    assert!(queue_item.unwrap().why.is_none());
}

#[test]
fn can_build_job_remotely() {
    setup();
    let jenkins = JenkinsBuilder::new(JENKINS_URL)
        .with_user("user", Some("password"))
        .build()
        .unwrap();

    let triggered = jenkins
        .job_builder("remote job")
        .unwrap()
        .remotely_with_token_and_cause("remote_token", None)
        .unwrap()
        .send();
    let triggered_ok = triggered.unwrap();

    let queue_item = triggered_ok.get_full_queue_item(&jenkins);
    assert!(queue_item.is_ok());
}

#[test]
fn can_get_build_with_git() {
    setup();
    let jenkins = JenkinsBuilder::new(JENKINS_URL)
        .with_user("user", Some("password"))
        .build()
        .unwrap();

    let job = jenkins.get_job("git triggered");
    assert!(job.is_ok());
    let build = jenkins.get_build("git triggered", 2);
    assert!(build.is_ok());
}

#[test]
fn can_get_matrix_job() {
    setup();
    let jenkins = JenkinsBuilder::new(JENKINS_URL)
        .with_user("user", Some("password"))
        .build()
        .unwrap();

    let job = jenkins.get_job("matrix job");
    assert!(job.is_ok());

    if let Ok(jenkins_api::Job::MatrixProject {
        active_configurations,
        ..
    }) = job
    {
        let config = active_configurations[0].get_full_job(&jenkins);
        assert!(config.is_ok());
    } else {
        assert!(false);
    }

    let build = jenkins.get_build("matrix job", 1);
    assert!(build.is_ok());

    if let Ok(jenkins_api::Build::MatrixBuild { runs, .. }) = build {
        assert!(runs[0].get_full_build(&jenkins).is_ok());
    } else {
        assert!(false);
    }
}

#[test]
fn can_build_job_with_parameters() {
    setup();
    let jenkins = JenkinsBuilder::new(JENKINS_URL)
        .with_user("user", Some("password"))
        .build()
        .unwrap();

    #[derive(Serialize)]
    struct Parameters {
        #[serde(rename = "bool-param")]
        bool_param: bool,
        #[serde(rename = "choose between")]
        choose_between: String,
        #[serde(rename = "free string param")]
        free_string_param: String,
    }

    let params = Parameters {
        bool_param: true,
        choose_between: "value2".to_string(),
        free_string_param: "my string param".to_string(),
    };

    let triggered = jenkins
        .job_builder("parameterized job")
        .unwrap()
        .with_parameters(&params)
        .unwrap()
        .send();
    assert!(triggered.is_ok());

    let queue_item = triggered.unwrap().get_full_queue_item(&jenkins);
    assert!(queue_item.is_ok());

    let queue_item_ok = queue_item.unwrap();

    let mut found_param1 = false;
    let mut found_param2 = false;
    let mut found_param3 = false;

    for action in queue_item_ok.actions {
        if let jenkins_api::action::Action::ParametersAction { parameters } = action {
            for param in parameters {
                if let jenkins_api::action::Parameter::BooleanParameterValue {
                    value: true, ..
                } = param
                {
                    found_param1 = true;
                }
                if let jenkins_api::action::Parameter::StringParameterValue { value, .. } = param {
                    if value == params.choose_between {
                        found_param2 = true;
                    }
                    if value == params.free_string_param {
                        found_param3 = true;
                    }
                }
            }
        }
    }

    assert!(found_param1);
    assert!(found_param2);
    assert!(found_param3);
}

#[test]
fn can_poll_scm() {
    setup();
    let jenkins = JenkinsBuilder::new(JENKINS_URL)
        .with_user("user", Some("password"))
        .build()
        .unwrap();

    let job = jenkins.get_job("git triggered");
    assert!(job.is_ok());

    let poll = job.unwrap().poll_scm(&jenkins);
    assert!(poll.is_ok());

    assert!(jenkins.poll_scm_job("git triggered").is_ok());
}

#[test]
fn can_get_maven_job() {
    setup();
    let jenkins = JenkinsBuilder::new(JENKINS_URL)
        .with_user("user", Some("password"))
        .build()
        .unwrap();

    let job = jenkins.get_job("maven job");
    assert!(job.is_ok());

    if let Ok(jenkins_api::Job::MavenModuleSet { modules, .. }) = job {
        let module = modules[0].get_full_job(&jenkins);
        if let Ok(jenkins_api::Job::MavenModule { last_build, .. }) = module {
            let build = last_build.unwrap().get_full_build(&jenkins);
            assert!(build.is_ok());
            if let Ok(jenkins_api::Build::MavenBuild {
                maven_artifacts, ..
            }) = build
            {
                let artifacts = maven_artifacts.get_full_artifact_record(&jenkins);
                assert!(artifacts.is_ok());
            } else {
                assert!(false);
            }
        } else {
            assert!(false);
        }
    } else {
        assert!(false);
    }

    let build = jenkins.get_build("maven job", 1);
    assert!(build.is_ok());

    if let Ok(jenkins_api::Build::MavenModuleSetBuild { .. }) = build {
        assert!(true);
    } else {
        assert!(false);
    }
}
