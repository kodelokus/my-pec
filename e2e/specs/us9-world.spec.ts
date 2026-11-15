import { button, next, submitKey, VALID_KEY, workspaceFile } from "../helpers";

describe("world: a world of its own", () => {
  before(async () => {
    await submitKey(VALID_KEY);
    await next(3);
    await button("Meet Pec").click();
    await expect($("p*=Hello, Sam! (")).toBeDisplayed();
  });

  it("opens the world from the main window and returns to the unsent text", async () => {
    await $("aria/Message Pec").setValue("a thought for later");
    await $("aria/World").click();
    await expect($("h1=Pec's world")).toBeDisplayed();
    const id = workspaceFile("IDENTITY.md").match(/- Id: (\S+)/)?.[1] ?? "";
    expect(id).toMatch(/^[2-9A-Z]{4}-[2-9A-Z]{4}-[2-9A-Z]{4}$/);
    // journey FR-033: the id stays recorded but is no longer shown.
    await expect($(`code=${id}`)).not.toBeExisting();
    // The home, whichever of the two the companion's DNA picked.
    await expect($('[aria-label^="Cottage, drawn"], [aria-label^="Burrow, drawn"]')).toBeExisting();

    await button("Back").click();
    await expect($("aria/Message Pec")).toHaveValue("a thought for later");
  });
});
