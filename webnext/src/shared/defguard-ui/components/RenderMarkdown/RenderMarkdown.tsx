import ReactMarkdown from 'react-markdown';

export const RenderMarkdown = ({ content }: { content?: string | null | undefined }) => {
  return <ReactMarkdown>{content}</ReactMarkdown>;
};
