import { button, REJECTED_KEY, submitKey, VALID_KEY } from "../helpers";

describe("onboarding US2: key problems", () => {
  it("explains a rejected key and accepts a valid one without restarting", async () => {
    await submitKey(REJECTED_KEY);
    await expect($("[role=alert]")).toHaveText(expect.stringContaining("didn't accept this key"));
    await expect(button("Next")).toBeDisabled();

    await $("aria/Claude API key").clearValue();
    await submitKey(VALID_KEY);
    await expect($("p*=Key accepted")).toBeDisplayed();
    await expect(button("Next")).toBeEnabled();
  });
});
