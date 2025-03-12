// //! bench()
// FIXME: inputs/passing/treeSort5k.ts failed: TypeError{"message":"Cannot subscript undefined"}
//        this doesn't happen in ValueScript - it's a regression

import BinaryTree from "../passing/helpers/BinaryTree.ts";
import randish from "../passing/helpers/randish.ts";
import Range from "../passing/helpers/Range.ts";

export default function main() {
  let tree = new BinaryTree<number>();

  for (const rand of Range.from(randish()).limit(5_000)) {
    tree.insert(Math.floor(4_000 * rand));
  }

  return [...tree];
}
