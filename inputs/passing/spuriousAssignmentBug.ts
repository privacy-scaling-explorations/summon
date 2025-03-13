// This was an SWC bug. It's fixed now that we've updated.

export default function main() {
  return [3n < "asdf", 3n >= "asdf"];
}
