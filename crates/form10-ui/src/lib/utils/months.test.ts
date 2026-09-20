import { describe, expect, it } from 'vitest';

import { monthOptions } from './months';

describe('monthOptions', () => {
  it('uses the financial year for fiscal month labels', () => {
    const months = monthOptions('2024-25');

    expect(months[8].short).toBe('DEC-24');
    expect(months[10].short).toBe('FEB-25');
  });

  it('uses plain month labels until the financial year is valid', () => {
    expect(monthOptions('')[0].short).toBe('Apr');
  });

  it('can be limited to the months parsed from the workbook', () => {
    const parsedMonths = [10, 11, 12];

    expect(monthOptions('2025-26').filter((month) => parsedMonths.includes(month.value))).toMatchObject([
      { short: 'JAN-26' },
      { short: 'FEB-26' },
      { short: 'MAR-26' }
    ]);
  });
});
