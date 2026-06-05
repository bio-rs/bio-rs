use super::super::types::{FixtureObservation, FixtureObservationIssue, VerificationIssueCode};
use crate::package::PackageFixture;
use std::collections::{HashMap, HashSet};

pub(super) type ObservationIndex<'a> = HashMap<&'a str, Vec<&'a FixtureObservation>>;

pub(super) enum FixtureObservationMatch<'a> {
    Missing,
    Unique(&'a FixtureObservation),
    Duplicate(&'a FixtureObservation),
}

pub(super) fn index_observations(observations: &[FixtureObservation]) -> ObservationIndex<'_> {
    let mut index = HashMap::new();
    for observation in observations {
        index
            .entry(observation.name.as_str())
            .or_insert_with(Vec::new)
            .push(observation);
    }
    index
}

pub(super) fn observation_for_fixture<'a>(
    fixture: &PackageFixture,
    observation_index: &ObservationIndex<'a>,
) -> FixtureObservationMatch<'a> {
    match observation_index
        .get(fixture.name.as_str())
        .map(Vec::as_slice)
    {
        None | Some([]) => FixtureObservationMatch::Missing,
        Some([observation]) => FixtureObservationMatch::Unique(observation),
        Some(observations) => FixtureObservationMatch::Duplicate(observations[0]),
    }
}

pub(super) fn unexpected_observation_issues(
    observation_index: &ObservationIndex<'_>,
    fixture_names: &HashSet<&str>,
) -> Vec<FixtureObservationIssue> {
    let mut issues = Vec::new();
    for (name, observations) in observation_index {
        if fixture_names.contains(name) {
            continue;
        }
        let code = if observations.len() > 1 {
            VerificationIssueCode::DuplicateObservation
        } else {
            VerificationIssueCode::UnexpectedObservation
        };
        issues.push(FixtureObservationIssue {
            code,
            name: (*name).to_string(),
            message: format!(
                "unexpected observation '{name}' is not declared by any package fixture"
            ),
        });
    }
    issues.sort_by(|left, right| left.name.cmp(&right.name));
    issues
}
