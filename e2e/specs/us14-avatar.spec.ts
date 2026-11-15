import { button, next, submitKey, VALID_KEY, workspaceFile, writeWorkspaceFile } from "../helpers";

// The avatar, with the end-to-end build's still portrait (no Live2D in it).
const docked = () => $(".dock [role=img]");

describe("avatar: the companion's character", () => {
  before(async () => {
    await submitKey(VALID_KEY);
    // avatar FR-028: the avatar step completes on its default with Next.
    await next(3);
    await button("Meet Pec").click();
  });

  it("takes the reply's mood, and the marker is never shown", async () => {
    await expect(docked()).toHaveAttribute("aria-label", expect.stringMatching(/^Pec/));
    await $("aria/Message Pec").setValue("I have good news!");
    await browser.keys("Enter");
    await expect($("p*=That's good news, Sam!")).toBeDisplayed();
    await expect(docked()).toHaveAttribute("aria-label", "Pec, happy");
    await expect($("body")).not.toHaveText(expect.stringContaining("<mood>"));
  });

  it("shows the portrait and says why when the animated character can't be shown", async () => {
    // avatar FR-031: a look edited by hand is honored. Its model is a stand-in, so whether or not this build has Live2D,
    // the character can't be drawn and settings say why.
    writeWorkspaceFile("avatar/pip/pip.model3.json", "{}");
    writeWorkspaceFile(
      "avatar/manifest.json",
      JSON.stringify({ version: 2, type: "live2d", character: "pip", model: "pip/pip.model3.json" }),
    );
    await browser.reloadSession();
    await expect($(".dock .figure svg.pip")).toBeDisplayed();
    await $("aria/Settings").click();
    await expect($("p*=so the still portrait is shown")).toBeDisplayed();
    await button("Back").click();
    await $("aria/Message Pec").setValue("Still there?");
    await browser.keys("Enter");
    await expect($$("p*=Hello, Sam! (")).toBeElementsArrayOfSize(2);
  });

  it("chooses the still portrait in settings", async () => {
    await $("aria/Settings").click();
    await $("aria/Still portrait").click();
    await browser.waitUntil(() => JSON.parse(workspaceFile("avatar/manifest.json")).type === "static");
    expect(JSON.parse(workspaceFile("avatar/manifest.json"))).toMatchObject({ character: "pip" });
    await expect($("p*=so the still portrait is shown")).not.toBeExisting();
    await button("Back").click();
  });

  it("floats on the desktop, can only open the chat, and goes when turned off", async () => {
    const main = await browser.getWindowHandle();
    await $("aria/Settings").click();
    await $("aria/Floating Pec").click();
    await browser.waitUntil(async () => (await browser.getWindowHandles()).length === 2);
    const floating = (await browser.getWindowHandles()).find((h) => h !== main) as string;
    await browser.switchToWindow(floating);
    // A new window starts blank and then loads the app's page.
    await browser.waitUntil(async () => (await browser.getUrl()) !== "about:blank");
    await expect($(".floating-view [role=img]")).toHaveAttribute("aria-label", expect.stringMatching(/^Pec/));
    // avatar FR-027: a main-window command is refused here.
    const refused = await browser.execute(() => {
      const internals = (
        window as never as { __TAURI_INTERNALS__: { invoke: (c: string, a: object) => Promise<unknown> } }
      ).__TAURI_INTERNALS__;
      return internals.invoke("chat_history", { before: null, limit: 1 }).then(
        () => false,
        () => true,
      );
    });
    expect(refused).toBe(true);
    await browser.switchToWindow(main);
    await $("aria/Floating Pec").click();
    await browser.waitUntil(async () => (await browser.getWindowHandles()).length === 1);
  });
});
