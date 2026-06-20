use tantivy::merge_policy::{MergeCandidate, MergePolicy};
use tantivy::SegmentMeta;

#[derive(Debug, Clone)]
pub struct TargetDocCountMergePolicy {
    pub target_num_docs: usize,
    pub max_merge_factor: usize,
    pub min_num_segments: usize,
}

impl MergePolicy for TargetDocCountMergePolicy {
    fn compute_merge_candidates(&self, segments: &[SegmentMeta]) -> Vec<MergeCandidate> {
        // Filter out segments already at or above target size
        let mut small_segments: Vec<&SegmentMeta> = segments
            .iter()
            .filter(|s| s.num_docs() < self.target_num_docs as u32)
            .collect();

        if small_segments.len() < self.min_num_segments {
            return Vec::new();
        }

        // Sort by num_docs ascending (merge smallest first)
        small_segments.sort_by_key(|s| s.num_docs());

        let mut candidates = Vec::new();
        let mut current_group: Vec<tantivy::SegmentId> = Vec::new();
        let mut current_docs: usize = 0;

        for seg in &small_segments {
            current_group.push(seg.id());
            current_docs += seg.num_docs() as usize;

            // If we've accumulated enough docs or hit max merge factor, finalize group
            if current_docs >= self.target_num_docs || current_group.len() >= self.max_merge_factor
            {
                if current_group.len() >= self.min_num_segments {
                    candidates.push(MergeCandidate(current_group));
                }
                current_group = Vec::new();
                current_docs = 0;
            }
        }

        // Handle remaining group
        if current_group.len() >= self.min_num_segments {
            candidates.push(MergeCandidate(current_group));
        }

        candidates
    }
}

impl Default for TargetDocCountMergePolicy {
    fn default() -> Self {
        Self {
            target_num_docs: 20_000_000,
            max_merge_factor: 10,
            min_num_segments: 2,
        }
    }
}
