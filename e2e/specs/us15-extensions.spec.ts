import { button, next, submitKey, VALID_KEY, workspaceFile, workspaceFiles } from "../helpers";

/** Waits for the window count, then returns the handle that isn't the main one. */
async function other(main: string, count: number): Promise<string | undefined> {
  await browser.waitUntil(async () => (await browser.getWindowHandles()).length === count);
  return (await browser.getWindowHandles()).find((h) => h !== main);
}

describe("extensions", () => {
  before(async () => {
    await submitKey(VALID_KEY);
    await next(3);
    await button("Meet Pec").click();
  });

  it("puts a reminder asked of the companion in the alarm list, with its mark", async () => {
    await $("aria/Message Pec").setValue("Please remind me every day at 7:30 to stretch");
    await browser.keys("Enter");
    // extensions FR-005: the change is shown on the reply.
    await expect($("button*=I set an alarm: 07:30 Stretch")).toBeDisplayed();
    expect(workspaceFile("alarms.md")).toContain("- [ ] every day 07:30 Stretch");
    // extensions FR-027: the first alarm brings the launch-at-login question, once.
    await expect($("p*=Alarms only ring while My Pec is running")).toBeDisplayed();
    await button("Not now").click();
    await $("aria/Alarms").click();
    await expect($("strong=07:30 Stretch")).toBeDisplayed();
    await button("Back").click();
  });

  it("opens a note made in the main window on the desktop, and keeps its text after a restart", async () => {
    const main = await browser.getWindowHandle();
    await $("aria/Sticky notes").click();
    await button("New note").click();
    const note = (await other(main, 2)) as string;
    await browser.switchToWindow(note);
    await browser.waitUntil(async () => (await browser.getUrl()) !== "about:blank");
    await $("textarea").setValue("Packing list");
    // extensions FR-014: saved without a save button.
    await browser.waitUntil(() =>
      workspaceFiles("notes").some(
        (f) => f.endsWith(".md") && workspaceFile(`notes/${f}`) === "Packing list",
      ),
    );
    await browser.switchToWindow(main);

    await browser.reloadSession();
    const restarted = await browser.getWindowHandle();
    const back = (await other(restarted, 2)) as string;
    await browser.switchToWindow(back);
    await browser.waitUntil(async () => (await browser.getUrl()) !== "about:blank");
    await expect($("textarea")).toHaveValue("Packing list");
    await browser.switchToWindow(restarted);
  });

  it("closes the notes when notes are turned off, and leaves their files", async () => {
    const main = await browser.getWindowHandle();
    await $("aria/Settings").click();
    await $("aria/Sticky notes").click();
    await other(main, 1);
    expect(workspaceFiles("notes").filter((f) => f.endsWith(".md"))).toHaveLength(1);
    await button("Back").click();
    await expect($("aria/Sticky notes")).not.toBeExisting();
  });
});
