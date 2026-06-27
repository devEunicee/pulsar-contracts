import React from "react";
import { render, screen, fireEvent } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import {
  PaymentHistoryFilter,
  DEFAULT_FILTER,
  FilterState,
} from "./PaymentHistoryFilter";

function makeValue(overrides: Partial<FilterState> = {}): FilterState {
  return { ...DEFAULT_FILTER, ...overrides };
}

describe("PaymentHistoryFilter", () => {
  it("renders all filter fields", () => {
    const onChange = jest.fn();
    render(
      <PaymentHistoryFilter value={makeValue()} onChange={onChange} />
    );
    expect(screen.getByLabelText("Start date")).toBeInTheDocument();
    expect(screen.getByLabelText("End date")).toBeInTheDocument();
    expect(screen.getByLabelText("Payment status filter")).toBeInTheDocument();
    expect(screen.getByPlaceholderText("Search by address…")).toBeInTheDocument();
    expect(screen.getByLabelText("Minimum amount")).toBeInTheDocument();
    expect(screen.getByLabelText("Maximum amount")).toBeInTheDocument();
    expect(screen.getByLabelText("Sort field")).toBeInTheDocument();
    expect(screen.getByLabelText("Sort order")).toBeInTheDocument();
  });

  it("calls onChange when dateStart changes", () => {
    const onChange = jest.fn();
    render(<PaymentHistoryFilter value={makeValue()} onChange={onChange} />);
    fireEvent.change(screen.getByLabelText("Start date"), {
      target: { value: "2024-01-01" },
    });
    expect(onChange).toHaveBeenCalledWith(
      expect.objectContaining({ dateStart: "2024-01-01" })
    );
  });

  it("calls onChange when status changes", () => {
    const onChange = jest.fn();
    render(<PaymentHistoryFilter value={makeValue()} onChange={onChange} />);
    fireEvent.change(screen.getByLabelText("Payment status filter"), {
      target: { value: "Completed" },
    });
    expect(onChange).toHaveBeenCalledWith(
      expect.objectContaining({ status: "Completed" })
    );
  });

  it("calls onChange when search input changes", async () => {
    const onChange = jest.fn();
    render(<PaymentHistoryFilter value={makeValue()} onChange={onChange} />);
    await userEvent.type(screen.getByPlaceholderText("Search by address…"), "G");
    expect(onChange).toHaveBeenCalled();
  });

  it("shows address suggestions when typing matches", () => {
    const onChange = jest.fn();
    const suggestions = ["GABC123", "GXYZ456"];
    render(
      <PaymentHistoryFilter
        value={makeValue({ search: "GAB" })}
        onChange={onChange}
        addressSuggestions={suggestions}
      />
    );
    const input = screen.getByPlaceholderText("Search by address…");
    fireEvent.focus(input);
    expect(screen.getByText("GABC123")).toBeInTheDocument();
    expect(screen.queryByText("GXYZ456")).not.toBeInTheDocument();
  });

  it("selects suggestion on mouse click", () => {
    const onChange = jest.fn();
    render(
      <PaymentHistoryFilter
        value={makeValue({ search: "G" })}
        onChange={onChange}
        addressSuggestions={["GABC123"]}
      />
    );
    fireEvent.focus(screen.getByPlaceholderText("Search by address…"));
    fireEvent.mouseDown(screen.getByText("GABC123"));
    expect(onChange).toHaveBeenCalledWith(
      expect.objectContaining({ search: "GABC123" })
    );
  });

  it("calls onChange when amountMin changes", () => {
    const onChange = jest.fn();
    render(<PaymentHistoryFilter value={makeValue()} onChange={onChange} />);
    fireEvent.change(screen.getByLabelText("Minimum amount"), {
      target: { value: "100" },
    });
    expect(onChange).toHaveBeenCalledWith(
      expect.objectContaining({ amountMin: "100" })
    );
  });

  it("calls onChange when sort field changes", () => {
    const onChange = jest.fn();
    render(<PaymentHistoryFilter value={makeValue()} onChange={onChange} />);
    fireEvent.change(screen.getByLabelText("Sort field"), {
      target: { value: "Amount" },
    });
    expect(onChange).toHaveBeenCalledWith(
      expect.objectContaining({ sortField: "Amount" })
    );
  });

  it("Clear all button is disabled when no active filters", () => {
    render(
      <PaymentHistoryFilter value={makeValue()} onChange={jest.fn()} />
    );
    expect(screen.getByLabelText("Clear all filters")).toBeDisabled();
  });

  it("Clear all button is enabled when filters differ from defaults", () => {
    render(
      <PaymentHistoryFilter
        value={makeValue({ status: "Completed" })}
        onChange={jest.fn()}
      />
    );
    expect(screen.getByLabelText("Clear all filters")).not.toBeDisabled();
  });

  it("calls onChange with DEFAULT_FILTER when Clear all is clicked", () => {
    const onChange = jest.fn();
    render(
      <PaymentHistoryFilter
        value={makeValue({ status: "Completed", dateStart: "2024-01-01" })}
        onChange={onChange}
      />
    );
    fireEvent.click(screen.getByLabelText("Clear all filters"));
    expect(onChange).toHaveBeenCalledWith(DEFAULT_FILTER);
  });

  it("has role=search on the form", () => {
    render(
      <PaymentHistoryFilter value={makeValue()} onChange={jest.fn()} />
    );
    expect(screen.getByRole("search")).toBeInTheDocument();
  });

  it("navigates suggestions with arrow keys", () => {
    const onChange = jest.fn();
    const suggestions = ["GABC123", "GXYZ456"];
    render(
      <PaymentHistoryFilter
        value={makeValue({ search: "G" })}
        onChange={onChange}
        addressSuggestions={suggestions}
      />
    );
    const input = screen.getByPlaceholderText("Search by address…");
    fireEvent.focus(input);
    fireEvent.keyDown(input, { key: "ArrowDown" });
    expect(screen.getByRole("option", { name: "GABC123" })).toHaveAttribute(
      "aria-selected",
      "true"
    );
  });
});
