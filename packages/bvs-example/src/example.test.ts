import { expect, test } from "vitest";

import { square } from "./example";

test("should square", async () => {
  const result = square(2);
  expect(result).toBe(4);
});
