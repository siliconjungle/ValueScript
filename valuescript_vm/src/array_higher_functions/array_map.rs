use crate::vs_value::ToVal;

use super::super::native_frame_function::NativeFrameFunction;
use super::super::vs_value::Val;
use super::array_mapping_frame::{ArrayMappingFrame, ArrayMappingState};

pub static MAP: NativeFrameFunction = NativeFrameFunction {
  make_frame: || Box::new(ArrayMappingFrame::new(Box::new(MapState::default()))),
};

#[derive(Default)]
struct MapState {
  map_results: Vec<Val>,
}

impl ArrayMappingState for MapState {
  fn process(&mut self, _i: usize, _element: &Val, mapped: Val) -> Option<Val> {
    self.map_results.push(mapped);
    return None;
  }

  fn finish(&mut self) -> Val {
    let mut map_results = Vec::new();
    std::mem::swap(&mut self.map_results, &mut map_results);
    map_results.to_val()
  }
}
