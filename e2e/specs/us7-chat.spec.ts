import { button, next, REPLAY_PORT, submitKey, VALID_KEY } from "../helpers";

const companion = () => $$("div.bubble.companion");
const sent = (text: string) => $(`//div[contains(@class, "bubble user") and contains(., "${text}")]`);

async function say(text: string) {
  await $("aria/Message Pec").setValue(text);
  await browser.keys("Enter");
}

describe("chat: talking with the companion", () => {
  before(async () => {
    await submitKey(VALID_KEY);
    await next(3);
    await button("Meet Pec").click();
  });

  it("greets once after setup, then answers a message", async () => {
    await expect($("p*=Hello, Sam! (")).toBeDisplayed();
    await say("Hi Pec");
    await expect(sent("Hi Pec")).toBeDisplayed();
    await expect(companion()).toBeElementsArrayOfSize(2);
  });

  it("keeps the conversation after a restart, with no second greeting", async () => {
    await browser.reloadSession();
    await expect(sent("Hi Pec")).toBeDisplayed();
    await expect(companion()).toBeElementsArrayOfSize(2);
  });

  it("stops a reply and regenerates it", async () => {
    await say("tell me a slow story");
    await expect($("p*=Once upon")).toBeDisplayed();
    await button("Stop").click();
    await expect($("span=Stopped")).toBeDisplayed();

    await button("Regenerate").click();
    await expect($("span=Stopped")).not.toBeExisting();
    await expect(companion()).toBeElementsArrayOfSize(3);
  });

  it("explains a key that is no longer accepted and keeps the message", async () => {
    await fetch(`http://127.0.0.1:${REPLAY_PORT}/__revoke`, { method: "POST" });
    await say("are you there?");
    await expect($("[role=alert]")).toHaveText(expect.stringContaining("no longer accepts your API key"));
    await expect(sent("are you there?")).toBeDisplayed();
  });
});
