import './style.scss';
import { useSearch } from '@tanstack/react-router';
import { m } from '../../paraglide/messages';
import { Page } from '../../shared/components/Page/Page';
import { Button } from '../../shared/defguard-ui/components/Button/Button';
import laptopImage from './assets/laptop.png';

export const OpenDesktopPage = () => {
  const { token } = useSearch({
    from: '/open-desktop',
  });

  const deepLinkUrl = new URL('defguard://addinstance');
  if (token) {
    deepLinkUrl.searchParams.set('token', token);
  }
  if (typeof window !== 'undefined') {
    deepLinkUrl.searchParams.set('url', window.location.origin);
  }

  return (
    <Page id="open-desktop-page" variant="default">
      <img src={laptopImage} />
      <h1>{m.open_desktop_title()}</h1>
      <p>{m.open_desktop_description()}</p>
      <a href={deepLinkUrl.toString()}>
        <Button className="open-desktop-cta" text={m.open_desktop_button()} />
      </a>
    </Page>
  );
};
