import DOMPurify from 'isomorphic-dompurify';
export default { hooks: { postprocess: (html: string) => DOMPurify.sanitize(html) } };
