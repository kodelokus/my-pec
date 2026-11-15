import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import * as api from "../../api/commands";
import type { Place } from "../../bindings/Place";
import type { WeatherSettings } from "../../bindings/WeatherSettings";
import { WeatherPanel } from "./WeatherPanel";

vi.mock("../../api/commands", async (original) => ({
  ...(await original<typeof api>()),
  weatherSettings: vi.fn(),
  weatherSearch: vi.fn(),
  weatherChoose: vi.fn(),
  weatherUnits: vi.fn(),
}));

const jakarta: Place = {
  name: "Jakarta",
  region: "Jakarta",
  country: "Indonesia",
  latitude: -6.21,
  longitude: 106.85,
};
const settings = (patch: Partial<WeatherSettings> = {}): WeatherSettings => ({
  place: null,
  units: "celsius",
  unreadable: false,
  updated: null,
  failing: false,
  ...patch,
});

beforeEach(() => {
  vi.mocked(api.weatherSettings).mockResolvedValue(settings());
});

const view = () => render(<WeatherPanel folder="C:\\data\\workspace\\weather.md" />);

// weather FR-005 to FR-007, FR-013.
describe("WeatherPanel", () => {
  it("finds a city, offers the matches with region and country, and chooses one", async () => {
    vi.mocked(api.weatherSearch).mockResolvedValue([jakarta, { ...jakarta, region: null, latitude: -6.2 }]);
    vi.mocked(api.weatherChoose).mockResolvedValue(settings({ place: jakarta, updated: "14:00" }));
    view();
    expect(await screen.findByText(/Choose a city to see its weather/)).toBeInTheDocument();
    await userEvent.type(screen.getByLabelText("City"), "Jakarta");
    await userEvent.click(screen.getByRole("button", { name: "Search" }));
    expect(api.weatherSearch).toHaveBeenCalledWith("Jakarta");
    const places = await screen.findByRole("list", { name: "Places found" });
    expect(places).toHaveTextContent("Jakarta, Jakarta, Indonesia");
    expect(places).toHaveTextContent("Jakarta, Indonesia");
    await userEvent.click(screen.getByRole("button", { name: "Jakarta, Jakarta, Indonesia" }));
    expect(api.weatherChoose).toHaveBeenCalledWith(jakarta);
    expect(
      await screen.findByText(/Showing the weather in Jakarta, Jakarta, Indonesia\. Updated 14:00\./),
    ).toBeInTheDocument();
  });

  it("says when nothing matched or the service couldn't be reached, without an alert", async () => {
    vi.mocked(api.weatherSearch).mockResolvedValueOnce([]).mockResolvedValueOnce(null);
    view();
    await userEvent.type(await screen.findByLabelText("City"), "Nowhere");
    await userEvent.click(screen.getByRole("button", { name: "Search" }));
    expect(await screen.findByRole("status")).toHaveTextContent("No place found");
    await userEvent.click(screen.getByRole("button", { name: "Search" }));
    expect(await screen.findByRole("status")).toHaveTextContent("couldn't reach the weather service");
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("switches units, and says plainly what is sent, a failed update and an unreadable place", async () => {
    vi.mocked(api.weatherSettings).mockResolvedValue(
      settings({ place: jakarta, failing: true, unreadable: true }),
    );
    vi.mocked(api.weatherUnits).mockResolvedValue(settings({ place: jakarta, units: "fahrenheit" }));
    view();
    expect(await screen.findByText(/Couldn't update the weather/)).toBeInTheDocument();
    expect(screen.getByText(/weather.md couldn't be read/)).toBeInTheDocument();
    expect(screen.getByText(/the city name you type is sent to Open-Meteo/)).toBeInTheDocument();
    expect(screen.getByText("Weather data by Open-Meteo.com")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("radio", { name: "Fahrenheit (°F)" }));
    expect(api.weatherUnits).toHaveBeenCalledWith("fahrenheit");
    expect(await screen.findByRole("radio", { name: "Fahrenheit (°F)" })).toBeChecked();
  });
});
