import { button, next, REJECTED_KEY, submitKey, VALID_KEY } from "../helpers";

describe("onboarding US6: key and model in settings", () => {
  before(async () => {
    await submitKey(VALID_KEY);
    await next(3);
    await button("Meet Pec").click();
    await $("aria/Settings").click();
  });

  it("keeps the old key when a replacement is rejected", async () => {
    await button("Replace key").click();
    await $("aria/Replace key").setValue(REJECTED_KEY);
    await button("Check key").click();
    await expect($("p*=didn't accept this key")).toBeDisplayed();
    await expect($("p*=ending in cdef")).toBeDisplayed();
  });

  it("keeps the model choice across a restart", async () => {
    const picker = $("#model-picker");
    await picker.selectByAttribute("value", "claude-sonnet-5-5");
    await browser.reloadSession();
    await $("aria/Settings").click();
    await expect($("#model-picker")).toHaveValue("claude-sonnet-5-5");
  });

  it("shows conversation as unavailable once the key is removed", async () => {
    await button("Remove key").click();
    await $$("button=Remove key")[1].click();
    await button("Back").click();
    await expect($("p*=until you add an API key")).toBeDisplayed();
    await expect($("aria/Setup steps")).not.toBeExisting();
  });
});
