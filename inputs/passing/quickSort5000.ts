//! bench()

import quickSort from "./helpers/quickSort.ts";
import randish from "./helpers/randish.ts";
import { Range_from } from "./helpers/range.ts";

export default function main() {
  let nums = [
    ...Range_from(randish())
      .map((x) => Math.floor(5_000 * x))
      .limit(5_000),
  ];

  nums = quickSort(nums, (a, b) => a - b);

  return nums;
}
