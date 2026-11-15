import { button, next, submitKey, VALID_KEY, workspaceFile } from "../helpers";

const sent = (text: string) => $(`//div[contains(@class, "bubble user") and contains(., "${text}")]`);

async function say(text: string) {
  await $("aria/Message Pec").setValue(text);
  await browser.keys("Enter");
}

describe("compaction: a long conversation compacts and goes on", () => {
  before(async () => {
    await submitKey(VALID_KEY);
    await next(3);
    await button("Meet Pec").click();
    await expect($("p*=Hello, Sam! (")).toBeDisplayed();
  });

  it("summarizes the older part after the reply that made it due, with a divider", async () => {
    await say("make it long");
    await expect($("p*=That is a lot to take in.")).toBeDisplayed();
    const divider = button("Earlier conversation summarized");
    await divider.waitForDisplayed({ timeout: 10_000 });
    await divider.click();
    await expect($("p*=Sam shared a long text")).toBeDisplayed();
    expect(workspaceFile("sessions/main.jsonl")).toContain('"type":"compaction"');
  });

  it("keeps talking after the compaction, with every earlier message still there", async () => {
    await say("still with me?");
    await expect(sent("still with me?")).toBeDisplayed();
    const replies = $$("div.bubble.companion");
    await expect(replies).toBeElementsArrayOfSize(3);
    await expect(sent("make it long")).toBeDisplayed();
    // The greeting, from before the compaction, is still the first reply.
    await expect(replies[0]).toHaveText(expect.stringContaining("Hello, Sam! ("));
  });
});
