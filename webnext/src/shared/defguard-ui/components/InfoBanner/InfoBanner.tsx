import { useMemo } from 'react';
import { Icon } from '../Icon';
import type { IconKindValue } from '../Icon/icon-types';
import './style.scss';
import clsx from 'clsx';
import { RenderMarkdown } from '../RenderMarkdown/RenderMarkdown';

type Props = {
  icon: IconKindValue;
  text: string | string[];
  variant?: 'info' | 'warning';
  markdown?: boolean;
};

export const InfoBanner = ({ icon, text, markdown = false, variant = 'info' }: Props) => {
  const textContents = useMemo(() => {
    if (markdown) {
      const prop = Array.isArray(text) ? text.join('\n') : text;
      return <RenderMarkdown content={prop} />;
    }
    if (Array.isArray(text)) {
      return text.map((content, index) => <p key={index}>{content}</p>);
    }
    return <p>{text}</p>;
  }, [text, markdown]);

  return (
    <div className="info-banner spacer">
      <div className={clsx('inner', `variant-${variant}`)}>
        <div className="icon-track">
          <Icon icon={icon} size={20} />
        </div>
        <div className="content">{textContents}</div>
      </div>
    </div>
  );
};
