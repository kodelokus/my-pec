import { button, next, submitKey, VALID_KEY, workspaceFile } from "../helpers";

/** The weather line under the character, read even while it is empty. */
const weatherLine = () => browser.execute(() => document.querySelector(".weather-line")?.textContent ?? null);

// Weather is off until turned on; a chosen city's weather is drawn and said beside the character.
describe("weather", () => {
  before(async () => {
    await submitKey(VALID_KEY);
    await next(3);
    await button("Meet Pec").click();
  });

  it("shows nothing until Weather is turned on and a city is chosen", async () => {
    await expect($("aria/Message Pec")).toBeDisplayed();
    expect(await weatherLine()).toBe("");
    await expect($(".weather-sky")).not.toBeExisting();
  });

  it("finds the city, shows its weather around the character, and goes when turned off", async () => {
    await $("aria/Settings").click();
    await $("aria/Weather").click();
    await $("aria/City").setValue("Jakarta");
    await button("Search").click();
    await button("Jakarta, Jakarta, Indonesia").click();
    await expect($("p*=Showing the weather in Jakarta, Jakarta, Indonesia.")).toBeDisplayed();
    expect(workspaceFile("weather.md")).toContain("- Place: Jakarta, Jakarta, Indonesia");
    await button("Back").click();

    await browser.waitUntil(async () => (await weatherLine()) === "Rain · 26°");
    await expect($(".weather-sky")).toBeExisting();

    await $("aria/Settings").click();
    await $("aria/Weather").click();
    await button("Back").click();
    await browser.waitUntil(async () => (await weatherLine()) === "");
    await expect($(".weather-sky")).not.toBeExisting();
    // weather FR-004: the place is kept for next time.
    expect(workspaceFile("weather.md")).toContain("- Place: Jakarta, Jakarta, Indonesia");
  });
});
