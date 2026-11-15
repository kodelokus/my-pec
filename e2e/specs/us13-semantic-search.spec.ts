import { button, next, submitKey, VALID_KEY, writeWorkspaceFile } from "../helpers";

describe("search: search by meaning is offered, never forced", () => {
  before(async () => {
    await submitKey(VALID_KEY);
    await next(3);
    await button("Meet Pec").click();
    await expect($("p*=Hello, Sam! (")).toBeDisplayed();
  });

  it("offers the download in settings with its size and source, and downloads nothing by itself", async () => {
    await $("aria/Settings").click();
    await expect($("h2=Search by meaning")).toBeDisplayed();
    await expect($("p*=one-time download of a small model (135 MB)")).toBeDisplayed();
    await expect($("p*=huggingface.co")).toBeDisplayed();
    await expect(button("Download")).toBeDisplayed();
    // No model, so no switch and nothing to remove.
    await expect($("aria/Find memories by meaning")).not.toBeExisting();
    await expect(button("Remove the model")).not.toBeExisting();
    await button("Back").click();
  });

  it("lets the memory browser's offer be put away while keyword search works as before", async () => {
    writeWorkspaceFile("memory/2026-11-01.md", "# 2026-11-01\n\n- 09:00 Allergic to peanuts.\n");
    await $("aria/Memory").click();
    await expect($("h1=Memory")).toBeDisplayed();
    const offer = $("aside[aria-label='Search by meaning']");
    await expect(offer).toBeDisplayed();
    await offer.$("button=Not now").click();
    await expect(offer).not.toBeExisting();

    await $("aria/Search memory").setValue("peanuts");
    await button("Search").click();
    await expect($("button*=Allergic to peanuts.")).toBeDisplayed();
  });
});
