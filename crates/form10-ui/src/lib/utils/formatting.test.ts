import { describe, expect, it } from 'vitest';

import { fileName, formatCurrency } from './formatting';

describe('formatting', () => {
  it('formats Indian rupees for export metrics', () => {
    expect(formatCurrency(12345.5)).toBe('₹12,345.50');
  });

  it('extracts a file name from either path separator', () => {
    expect(fileName('/exports/FORM-10.xlsx')).toBe('FORM-10.xlsx');
    expect(fileName('C:\\exports\\FORM-10.xlsx')).toBe('FORM-10.xlsx');
  });
});
