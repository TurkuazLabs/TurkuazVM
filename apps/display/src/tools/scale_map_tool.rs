// # 📄 Dosya Yolu: /turkuazvm/apps/display/src/tools/scale_map_tool.rs
// # 📌 Amac: Native display nearest-neighbor olceklendirmesi icin tekrar kullanilan eksen eslemelerini hazirlar
// # 📌 Modul - Rust
// # Version: 0.40.2
// # Aciklama: Her render pikselinde bolme yapmak yerine guest/host boyutu degisene kadar x ve y source index tablolarini cache eder
// # Bagimli Oldugu Katman: Tool | View

#[derive(Default)]
pub struct ScaleMapTool {
    source_width: usize,
    source_height: usize,
    target_width: usize,
    target_height: usize,
    x_map: Vec<usize>,
    y_map: Vec<usize>,
}

impl ScaleMapTool {
    pub fn ensure(
        &mut self,
        source_width: usize,
        source_height: usize,
        target_width: usize,
        target_height: usize,
    ) {
        if self.source_width == source_width
            && self.source_height == source_height
            && self.target_width == target_width
            && self.target_height == target_height
        {
            return;
        }
        self.source_width = source_width;
        self.source_height = source_height;
        self.target_width = target_width;
        self.target_height = target_height;
        self.x_map = build_axis_map(source_width, target_width);
        self.y_map = build_axis_map(source_height, target_height);
    }

    pub fn x_map(&self) -> &[usize] {
        &self.x_map
    }

    pub fn y_map(&self) -> &[usize] {
        &self.y_map
    }
}

fn build_axis_map(source: usize, target: usize) -> Vec<usize> {
    if source == 0 || target == 0 {
        return Vec::new();
    }
    (0..target)
        .map(|position| (position * source / target).min(source.saturating_sub(1)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::build_axis_map;

    #[test]
    fn axis_map_keeps_source_bounds() {
        let map = build_axis_map(640, 1920);
        assert_eq!(map.len(), 1920);
        assert_eq!(map.first().copied(), Some(0));
        assert_eq!(map.last().copied(), Some(639));
    }
}
