import React from "react";
import { render, screen, fireEvent } from "@testing-library/react";
import { renderHook, act } from "@testing-library/react";
import { Pagination, usePagination } from "./Pagination";

function makeProps(overrides = {}) {
  return {
    currentCursor: null,
    nextCursor: "cursor_abc",
    prevCursors: [],
    pageNumber: 1,
    onNext: jest.fn(),
    onPrev: jest.fn(),
    onFirst: jest.fn(),
    ...overrides,
  };
}

describe("Pagination", () => {
  it("renders Prev and Next buttons", () => {
    render(<Pagination {...makeProps()} />);
    expect(screen.getByLabelText("Go to previous page")).toBeInTheDocument();
    expect(screen.getByLabelText("Go to next page")).toBeInTheDocument();
  });

  it("disables Prev and First on page 1", () => {
    render(<Pagination {...makeProps({ pageNumber: 1, prevCursors: [] })} />);
    expect(screen.getByLabelText("Go to previous page")).toBeDisabled();
    expect(screen.getByLabelText("Go to first page")).toBeDisabled();
  });

  it("disables Next when nextCursor is null", () => {
    render(<Pagination {...makeProps({ nextCursor: null })} />);
    expect(screen.getByLabelText("Go to next page")).toBeDisabled();
  });

  it("enables Prev when pageNumber > 1", () => {
    render(
      <Pagination
        {...makeProps({ pageNumber: 2, prevCursors: [null] })}
      />
    );
    expect(screen.getByLabelText("Go to previous page")).not.toBeDisabled();
  });

  it("calls onNext with nextCursor when Next is clicked", () => {
    const onNext = jest.fn();
    render(
      <Pagination {...makeProps({ onNext, nextCursor: "cursor_abc" })} />
    );
    fireEvent.click(screen.getByLabelText("Go to next page"));
    expect(onNext).toHaveBeenCalledWith("cursor_abc");
  });

  it("calls onPrev when Prev is clicked", () => {
    const onPrev = jest.fn();
    render(
      <Pagination {...makeProps({ onPrev, pageNumber: 2, prevCursors: [null] })} />
    );
    fireEvent.click(screen.getByLabelText("Go to previous page"));
    expect(onPrev).toHaveBeenCalled();
  });

  it("calls onFirst when first-page button is clicked", () => {
    const onFirst = jest.fn();
    render(
      <Pagination {...makeProps({ onFirst, pageNumber: 3, prevCursors: [null, "c1"] })} />
    );
    fireEvent.click(screen.getByLabelText("Go to first page"));
    expect(onFirst).toHaveBeenCalled();
  });

  it("shows page number indicator", () => {
    render(<Pagination {...makeProps({ pageNumber: 3 })} />);
    expect(screen.getByText("Page 3")).toBeInTheDocument();
  });

  it("has aria-live on page indicator", () => {
    render(<Pagination {...makeProps()} />);
    const indicator = screen.getByText(/Page 1/);
    expect(indicator).toHaveAttribute("aria-live", "polite");
  });

  it("has nav role with aria-label", () => {
    render(<Pagination {...makeProps()} />);
    expect(screen.getByRole("navigation", { name: "Pagination" })).toBeInTheDocument();
  });
});

describe("usePagination", () => {
  it("starts on page 1 with null cursor", () => {
    const { result } = renderHook(() => usePagination());
    expect(result.current[0].cursor).toBeNull();
    expect(result.current[0].pageNumber).toBe(1);
  });

  it("goNext increments page and updates cursor", () => {
    const { result } = renderHook(() => usePagination());
    act(() => result.current[1].goNext("cursor_2"));
    expect(result.current[0].cursor).toBe("cursor_2");
    expect(result.current[0].pageNumber).toBe(2);
    expect(result.current[0].prevCursors).toEqual([null]);
  });

  it("goPrev decrements page and restores previous cursor", () => {
    const { result } = renderHook(() => usePagination());
    act(() => result.current[1].goNext("cursor_2"));
    act(() => result.current[1].goPrev());
    expect(result.current[0].cursor).toBeNull();
    expect(result.current[0].pageNumber).toBe(1);
  });

  it("goFirst resets to page 1", () => {
    const { result } = renderHook(() => usePagination());
    act(() => result.current[1].goNext("cursor_2"));
    act(() => result.current[1].goNext("cursor_3"));
    act(() => result.current[1].goFirst());
    expect(result.current[0].cursor).toBeNull();
    expect(result.current[0].pageNumber).toBe(1);
    expect(result.current[0].prevCursors).toHaveLength(0);
  });

  it("goPrev does nothing on page 1", () => {
    const { result } = renderHook(() => usePagination());
    act(() => result.current[1].goPrev());
    expect(result.current[0].pageNumber).toBe(1);
  });
});
