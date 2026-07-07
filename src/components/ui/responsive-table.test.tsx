import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ResponsiveTable, TableView, CardView } from './responsive-table';

describe('ResponsiveTable primitives', () => {
  it('renders both children in the DOM so CSS can gate visibility per breakpoint', () => {
    render(
      <ResponsiveTable>
        <TableView>
          <table>
            <tbody>
              <tr>
                <td data-testid="desktop-row">desktop</td>
              </tr>
            </tbody>
          </table>
        </TableView>
        <CardView>
          <ul>
            <li data-testid="mobile-row">mobile</li>
          </ul>
        </CardView>
      </ResponsiveTable>,
    );
    // Both content nodes are present — the responsive contract is CSS-only.
    expect(screen.getByTestId('desktop-row')).toBeInTheDocument();
    expect(screen.getByTestId('mobile-row')).toBeInTheDocument();
  });

  it('TableView wraps its children in a container hidden below md', () => {
    render(
      <TableView data-testid="wrap">
        <span>content</span>
      </TableView>,
    );
    const wrap = screen.getByTestId('wrap');
    // Class must include `hidden` and `md:block` — that's the contract callers
    // rely on to gate table density to desktop.
    expect(wrap.className).toContain('hidden');
    expect(wrap.className).toContain('md:block');
  });

  it('CardView wraps its children in a container hidden at md+', () => {
    render(
      <CardView data-testid="wrap">
        <span>content</span>
      </CardView>,
    );
    const wrap = screen.getByTestId('wrap');
    expect(wrap.className).toContain('md:hidden');
  });

  it('ResponsiveTable forwards className and data-slot', () => {
    render(
      <ResponsiveTable className="custom-shell" data-testid="root">
        <span>x</span>
      </ResponsiveTable>,
    );
    const root = screen.getByTestId('root');
    expect(root.className).toContain('custom-shell');
    expect(root).toHaveAttribute('data-slot', 'responsive-table');
  });
});
