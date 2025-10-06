export const openVirtualLink = (value?: string): void => {
  if (!value) return;

  const anchorElement = document.createElement('a');
  anchorElement.style.display = 'none';
  anchorElement.href = value;
  anchorElement.target = '_blank';
  anchorElement.rel = 'noopener noreferrer';
  document.body.appendChild(anchorElement);
  anchorElement.click();
  anchorElement.remove();
};
