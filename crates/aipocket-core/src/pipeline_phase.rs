use serde::{Deserialize, Serialize};

/// Last completed durable pipeline stage. Stored in `runs.phase`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PipelinePhase {
    #[default]
    Started,
    Discovery,
    Extract,
    Probe,
    Gpt,
    Validate,
    Finalize,
    Finished,
}

impl PipelinePhase {
    pub const ORDER: [Self; 8] = [
        Self::Started,
        Self::Discovery,
        Self::Extract,
        Self::Probe,
        Self::Gpt,
        Self::Validate,
        Self::Finalize,
        Self::Finished,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Started => "started",
            Self::Discovery => "discovery",
            Self::Extract => "extract",
            Self::Probe => "probe",
            Self::Gpt => "gpt",
            Self::Validate => "validate",
            Self::Finalize => "finalize",
            Self::Finished => "finished",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ORDER
            .iter()
            .copied()
            .find(|phase| phase.as_str() == value)
    }

    pub fn rank(self) -> usize {
        Self::ORDER
            .iter()
            .position(|phase| *phase == self)
            .unwrap_or(0)
    }

    /// Resume cursor after applying the pre-ADR lie: `validate`/`finalize` with
    /// hits but no candidates means extract never spilled.
    pub fn reconcile(stored: Option<&str>, discovery_hits: u64, candidates: u64) -> Self {
        let phase = stored.and_then(Self::parse).unwrap_or(Self::Started);
        if matches!(phase, Self::Validate | Self::Finalize) && candidates == 0 && discovery_hits > 0
        {
            return Self::Discovery;
        }
        phase
    }
}

#[cfg(test)]
mod tests {
    use super::PipelinePhase;

    #[test]
    fn order_matches_documented_pipeline() {
        let names: Vec<_> = PipelinePhase::ORDER.iter().map(|p| p.as_str()).collect();
        assert_eq!(
            names,
            [
                "started",
                "discovery",
                "extract",
                "probe",
                "gpt",
                "validate",
                "finalize",
                "finished"
            ]
        );
        assert!(PipelinePhase::Extract < PipelinePhase::Probe);
        assert!(PipelinePhase::Gpt < PipelinePhase::Validate);
        assert_eq!(PipelinePhase::parse("unknown"), None);
        assert_eq!(
            PipelinePhase::parse("extract"),
            Some(PipelinePhase::Extract)
        );
        assert!(PipelinePhase::Finished.rank() > PipelinePhase::Validate.rank());
    }

    #[test]
    fn reconcile_rewinds_lying_validate_not_empty_extract() {
        assert_eq!(
            PipelinePhase::reconcile(Some("validate"), 3, 0),
            PipelinePhase::Discovery
        );
        assert_eq!(
            PipelinePhase::reconcile(Some("finalize"), 2, 0),
            PipelinePhase::Discovery
        );
        assert_eq!(
            PipelinePhase::reconcile(Some("validate"), 3, 1),
            PipelinePhase::Validate
        );
        assert_eq!(
            PipelinePhase::reconcile(Some("extract"), 4, 0),
            PipelinePhase::Extract
        );
        assert_eq!(
            PipelinePhase::reconcile(Some("unknown"), 9, 0),
            PipelinePhase::Started
        );
        assert_eq!(PipelinePhase::reconcile(None, 0, 0), PipelinePhase::Started);
    }
}
