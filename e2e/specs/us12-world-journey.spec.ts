import { button, next, submitKey, VALID_KEY, workspaceFile } from "../helpers";

describe("journey: the world maps the user's days", () => {
  before(async () => {
    await submitKey(VALID_KEY);
    await next(3);
    await button("Meet Pec").click();
    await expect($("p*=Hello, Sam! (")).toBeDisplayed();
  });

  it("draws the day during the dream and tells its story in the world", async () => {
    await $("aria/Message Pec").setValue("Just back from a 10 km run before work!");
    await browser.keys("Enter");
    await $('//div[contains(@class, "bubble user") and contains(., "10 km run")]').waitForDisplayed();
    // Tomorrow, as far as the app knows: today has ended, so the dream names it.
    await browser.execute(() => {
      const Real = Date;
      const day = 24 * 60 * 60 * 1000;
      class Tomorrow extends Real {
        constructor(...args: [] | [number]) {
          if (args.length === 0) super(Real.now() + day);
          else super(args[0]);
        }
        static now() {
          return Real.now() + day;
        }
      }
      Object.assign(window, { Date: Tomorrow });
    });
    await $("aria/Dreams").click();
    await button("Dream now").click();
    await expect($("p*=I dreamt of a long road")).toBeDisplayed();
    await button("Back").click();

    await $("aria/World").click();
    await expect($('//div[contains(@class, "bubble voice") and contains(., "“10 km run”")]')).toBeDisplayed();
    expect(workspaceFile("world/journey.jsonl")).toContain('"keyword":"10 km run"');
    // The list tells the story beside the object, and selecting it tells it again.
    const story = $('//li[contains(., "— “10 km run”")]');
    await story.$("button.link").click();
    await expect(
      $('//*[@role="status" and contains(., "You ran ten kilometres before work.")]'),
    ).toBeDisplayed();
    // Its tooltip on the map carries the day's keywords.
    await $('.world-item[aria-pressed="true"]').moveTo();
    await expect($("[role=tooltip]")).toHaveText(expect.stringContaining("10 km run"));
    await story.$("summary=Why this one").click();
    await expect(story.$("td*=(drawn)")).toBeDisplayed();
  });

  it("forgets a story and keeps its drawing", async () => {
    const world = workspaceFile("world/world.jsonl");
    await $('//li[contains(., "— “10 km run”")]').$("button=Forget this story").click();
    await $("[role=alertdialog]").$("button=Forget").click();
    await expect($("p=The story was forgotten. The drawing stays.")).toBeDisplayed();
    expect(workspaceFile("world/journey.jsonl")).not.toContain("10 km run");
    expect(workspaceFile("world/world.jsonl")).toBe(world);
  });
});
