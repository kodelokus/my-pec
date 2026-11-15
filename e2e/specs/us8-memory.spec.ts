import { button, next, submitKey, VALID_KEY, workspaceFile } from "../helpers";

async function say(text: string) {
  await $("aria/Message Pec").setValue(text);
  await browser.keys("Enter");
}

describe("memory: the companion keeps what matters", () => {
  before(async () => {
    await submitKey(VALID_KEY);
    await next(3);
    await button("Meet Pec").click();
    await expect($("p*=Hello, Sam! (")).toBeDisplayed();
  });

  it("remembers on request and shows it on the reply", async () => {
    await say("Please remember that I'm allergic to peanuts.");
    await expect($("p*=No peanuts, ever.")).toBeDisplayed();
    await expect($("aria/Saved to memory")).toHaveText(
      expect.stringContaining("I'll remember: Allergic to peanuts."),
    );
    expect(workspaceFile("MEMORY.md")).toContain("- Allergic to peanuts. <!-- id:m_");
  });

  it("opens the memory browser from the mark and edits the profile", async () => {
    await button("I'll remember: Allergic to peanuts.").click();
    await expect($("h1=Memory")).toBeDisplayed();
    await expect($("span=Allergic to peanuts.")).toBeDisplayed();

    await button("Your profile").click();
    await button("Edit").click();
    await $("textarea").setValue("# About the user\n\n- Call them: Sam\n- Lives in: Bandung\n");
    await button("Save").click();
    await expect($("p=Saved.")).toBeDisplayed();
    expect(workspaceFile("USER.md")).toContain("- Lives in: Bandung");
  });

  it("forgets an entry after confirming", async () => {
    await button("Long-term memory").click();
    await $("aria/Forget: Allergic to peanuts.").click();
    await expect($("[role=alertdialog]")).toBeDisplayed();
    await $("[role=alertdialog]").$("button=Forget").click();
    await expect($("p=Forgotten.")).toBeDisplayed();
    expect(workspaceFile("MEMORY.md")).not.toContain("peanuts");
  });
});
