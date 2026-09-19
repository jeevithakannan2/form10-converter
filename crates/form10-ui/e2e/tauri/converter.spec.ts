describe('Form 10 Converter desktop application', () => {
  it('renders the native webview at the import stage', async () => {
    await expect($('h1=Form 10 Converter')).toBeDisplayed();
    await expect($('h2=Import workbook')).toBeDisplayed();
    await expect($('button=Continue to details')).not.toBeExisting();
  });

  it('exposes the native workbook picker action', async () => {
    const importButton = await $('button*=Choose Excel file');
    await expect(importButton).toBeClickable();
  });
});
