import { Folder, Camera, Heart, Settings, type LucideProps } from 'lucide-react';

const iconMap: Record<string, React.FC<LucideProps>> = {
    camera: Camera,
    heart: Heart,
    settings: Settings,
    folder: Folder,
};

// @ts-ignore
const DynamicIcon = ({ name, ...props }: { name: string } & LucideProps) => {
    if (!name) return null;
    const Icon = iconMap[name];
    return <Icon {...props} />;
};