import { test } from "node:test";
import assert from "node:assert/strict";
import {
  createPendingRequest,
  rejectPendingRequest,
  resolvePendingRequest,
} from "./requestManagerCore.ts";

test("pending requests reject after the configured timeout", async () => {
  const request = createPendingRequest<string>(10);
  await assert.rejects(request.promise, /Request timed out/);
});

test("cancelling a pending request removes it and rejects it", async () => {
  const request = createPendingRequest<string>(1000);
  request.cancel();
  await assert.rejects(request.promise, /Request cancelled/);
  assert.equal(resolvePendingRequest(request.requestId, "late"), false);
});

test("event results resolve and delete the pending request", async () => {
  const request = createPendingRequest<string>(1000);
  assert.equal(resolvePendingRequest(request.requestId, "done"), true);
  assert.equal(await request.promise, "done");
  assert.equal(rejectPendingRequest(request.requestId, new Error("late")), false);
});
