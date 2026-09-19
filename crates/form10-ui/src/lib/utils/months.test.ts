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
});
