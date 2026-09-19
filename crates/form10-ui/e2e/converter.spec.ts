import { expect, test } from '@playwright/test';

test('imports a workbook, validates settings, and completes an export', async ({ page }) => {
  await page.goto('/?mock');
  await expect(page.getByRole('heading', { name: 'Form 10 Converter' })).toBeVisible();
  await page.getByRole('button', { name: /Choose Excel file/i }).click();

  await expect(page.getByText('Milk-procurement.xlsx')).toBeVisible();
  await page.getByRole('button', { name: 'Continue to details' }).click();
  await expect(page.getByRole('heading', { name: 'Set up this register' })).toBeVisible();
  await page.getByRole('button', { name: 'Continue to export' }).click();
  await expect(page.getByText('₹4,764.00')).toBeVisible();

  await page.getByRole('button', { name: 'Choose location and create' }).click();
  await expect(page.getByRole('heading', { name: 'FORM-10 is ready' })).toBeVisible();
  await expect(page.getByText('FORM-10-2025-26.xlsx', { exact: true })).toBeVisible();
});

test('requires confirmation before replacing an existing workbook', async ({ page }) => {
  await page.goto('/?mock&existing-output');
  await page.getByRole('button', { name: /Choose Excel file/i }).click();
  await page.getByRole('button', { name: 'Continue to details' }).click();
  await page.getByRole('button', { name: 'Continue to export' }).click();
  await page.getByRole('button', { name: 'Choose location and create' }).click();

  await expect(page.getByRole('alertdialog')).toBeVisible();
  await page.getByRole('button', { name: 'Replace file' }).click();
  await expect(page.getByRole('heading', { name: 'FORM-10 is ready' })).toBeVisible();
});

test('reports invalid settings and supports appearance changes', async ({ page }) => {
  await page.goto('/?mock');
  await page.getByRole('button', { name: /Choose Excel file/i }).click();
  await page.getByRole('button', { name: 'Continue to details' }).click();
  await page.getByLabel('Financial year').fill('2025');
  await expect(page.getByRole('alert')).toContainText('Use a year like 2025-26.');
  await expect(page.getByRole('button', { name: 'Continue to export' })).toBeDisabled();

  await page.getByRole('button', { name: 'Toggle color appearance' }).click();
  await expect(page.locator('main')).toHaveClass(/dark/);
});
