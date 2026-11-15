describe("smoke", () => {
  it("opens the window on the wizard", async () => {
    await expect($("[role=status]")).toHaveText(expect.stringContaining("I need your own Claude API key"));
    await expect($("aria/Setup steps")).toBeDisplayed();
  });
});
