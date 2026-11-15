import { execFileSync, spawnSync } from "node:child_process";
import { APP, APP_ENV, button, next, submitKey, VALID_KEY } from "../helpers";

const instances = () =>
  execFileSync("tasklist", ["/FI", "IMAGENAME eq my-pec.exe", "/FO", "CSV", "/NH"], { encoding: "utf8" })
    .split("\n")
    .filter((line) => line.includes("my-pec.exe")).length;

/** Sends the window the same close request as its title-bar X (WM_CLOSE). */
function closeWindow() {
  const script = `
    Add-Type -Name W -Namespace U -MemberDefinition '[DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint m, IntPtr w, IntPtr l);'
    $p = Get-Process my-pec | Where-Object MainWindowTitle -eq "My Pec" | Select-Object -First 1
    [void][U.W]::PostMessage($p.MainWindowHandle, 0x10, [IntPtr]::Zero, [IntPtr]::Zero)`;
  execFileSync("powershell", ["-NoProfile", "-Command", script]);
}

/** The page inside a hidden window still reports itself as displayed, so ask Windows. The single-instance plugin owns a second window, titled differently. */
const windowVisible = () =>
  execFileSync(
    "powershell",
    ["-NoProfile", "-Command", "@(Get-Process my-pec | Where-Object MainWindowTitle -eq 'My Pec').Count"],
    { encoding: "utf8" },
  ).trim() !== "0";

describe("onboarding US5: tray and single instance", () => {
  before(async () => {
    await submitKey(VALID_KEY);
    await next(3);
    await button("Meet Pec").click();
    await expect($("p=Pec is here")).toBeDisplayed();
  });

  it("keeps one process when launched 20 more times (onboarding SC-008)", () => {
    for (let i = 0; i < 20; i++) spawnSync(APP, { env: { ...process.env, ...APP_ENV }, timeout: 10_000 });
    expect(instances()).toBe(1);
  });

  it("stays running when the window is closed", async () => {
    closeWindow();
    await expect($("[role=alertdialog]")).toHaveText(
      expect.stringContaining("right here in the system tray"),
    );
    await button("OK").click();
    await browser.waitUntil(() => !windowVisible());
    expect(instances()).toBe(1);
  });
});
