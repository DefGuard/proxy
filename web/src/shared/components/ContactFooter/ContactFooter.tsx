import './style.scss';
import { m } from '../../../paraglide/messages';

type Props = {
  email: string;
};

export const ContactFooter = ({ email }: Props) => {
  return (
    <p className="admin-contact">
      {m.footer_contact()} <a href={`mailto:${email}`}>{email}</a>
    </p>
  );
};
