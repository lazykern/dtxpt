use super::model::ChartEntry;

pub fn difficulty_rank(label: &str) -> usize {
    match label.to_ascii_uppercase().as_str() {
        "BASIC" | "NOVICE" | "BSC" | "BEGINNER" | "EASY" => 1,
        "ADVANCED" | "REGULAR" | "ADV" => 2,
        "EXTREME" | "EXPERT" | "EXT" => 3,
        "MASTER" | "MSTR" => 4,
        "DTXMANIA" => 5,
        _ => 99,
    }
}

fn chart_rank(chart: &ChartEntry) -> usize {
    let label_rank = difficulty_rank(&chart.label);
    if label_rank != 99 {
        return label_rank;
    }
    chart
        .path
        .file_stem()
        .map(|stem| difficulty_rank(&stem.to_string_lossy()))
        .filter(|rank| *rank != 99)
        .unwrap_or(99)
}

pub fn pick_chart_index(charts: &[ChartEntry], preferred: &str) -> usize {
    if charts.is_empty() {
        return 0;
    }
    if preferred.is_empty() {
        return 0;
    }
    if let Some(index) = charts
        .iter()
        .position(|chart| chart.label.eq_ignore_ascii_case(preferred))
    {
        return index;
    }

    let target = difficulty_rank(preferred);
    if target == 99 {
        return 0;
    }

    charts
        .iter()
        .enumerate()
        .map(|(index, chart)| {
            let rank = chart_rank(chart);
            let distance = rank.abs_diff(target);
            (distance, rank, index)
        })
        .min_by(|a, b| a.0.cmp(&b.0).then_with(|| b.1.cmp(&a.1)))
        .map(|(_, _, index)| index)
        .unwrap_or(0)
}

pub fn compare_difficulty_labels(a: &str, b: &str) -> std::cmp::Ordering {
    difficulty_rank(a).cmp(&difficulty_rank(b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn chart(label: &str, stem: &str) -> ChartEntry {
        ChartEntry {
            path: PathBuf::from(format!("{stem}.dtx")),
            label: label.to_string(),
            level: None,
            bgm_path: None,
            bgm_volume: 100,
        }
    }

    #[test]
    fn picks_exact_label_match() {
        let charts = vec![
            chart("BASIC", "bsc"),
            chart("ADVANCED", "adv"),
            chart("MASTER", "mstr"),
        ];
        assert_eq!(pick_chart_index(&charts, "ADVANCED"), 1);
    }

    #[test]
    fn picks_closest_rank_when_label_missing() {
        let charts = vec![chart("BASIC", "bsc"), chart("ADVANCED", "adv")];
        assert_eq!(pick_chart_index(&charts, "MASTER"), 1);
    }

    #[test]
    fn picks_highest_available_below_preferred_on_tie() {
        let charts = vec![chart("BASIC", "bsc"), chart("EXTREME", "ext")];
        assert_eq!(pick_chart_index(&charts, "MASTER"), 1);
    }

    #[test]
    fn matches_difficulty_from_filename_stem() {
        let charts = vec![
            chart("DTX", "bsc"),
            chart("DTX", "adv"),
            chart("DTX", "mstr"),
        ];
        assert_eq!(pick_chart_index(&charts, "MASTER"), 2);
    }
}
