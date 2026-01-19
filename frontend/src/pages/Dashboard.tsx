import React, { useEffect, useState } from 'react';
import { api } from '../services';
import type { Device, CreateDeviceRequest, UpdateDeviceRequest } from '../types';
import { Card, CardHeader, CardTitle, CardContent, CardFooter } from '../components/ui/card';
import { Button } from '../components/ui/button';
import { Input } from '../components/ui/input';
import { Label } from '../components/ui/label';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger, DialogFooter } from '../components/ui/dialog';
import { Power, Trash2, Plus, Monitor, Server, Laptop, Pencil } from 'lucide-react';
import { useAuth } from '../context/AuthContext';
import { toast } from 'sonner';

export default function Dashboard() {
    const { user } = useAuth();
    const [devices, setDevices] = useState<Device[]>([]);
    const [isCreateOpen, setIsCreateOpen] = useState(false);
    const [isEditOpen, setIsEditOpen] = useState(false);
    
    // New device form
    const [newDevice, setNewDevice] = useState<CreateDeviceRequest>({
        name: '',
        mac_address: '',
        ip_address: '',
        broadcast_addr: '',
        icon: 'desktop'
    });

    // Edit device form
    const [editingDevice, setEditingDevice] = useState<(UpdateDeviceRequest & { id: number }) | null>(null);
    const [formErrors, setFormErrors] = useState<Record<string, string>>({});

    const validateDevice = (data: { name?: string, mac_address?: string, ip_address?: string, broadcast_addr?: string }) => {
        const errors: Record<string, string> = {};
        if (!data.name?.trim()) errors.name = "Name is required";
        if (!data.mac_address?.trim()) {
            errors.mac_address = "MAC Address is required";
        } else {
            const macRegex = /^([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})$/;
            if (!macRegex.test(data.mac_address)) errors.mac_address = "Invalid MAC Address format (e.g., AA:BB:CC:DD:EE:FF)";
        }
    
        const ipRegex = /^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$/;
        
        if (data.ip_address && data.ip_address.trim() !== '' && !ipRegex.test(data.ip_address)) {
            errors.ip_address = "Invalid IP Address format";
        }
    
        if (data.broadcast_addr && data.broadcast_addr.trim() !== '' && !ipRegex.test(data.broadcast_addr)) {
            errors.broadcast_addr = "Invalid Broadcast Address format";
        }
    
        return errors;
    };

    useEffect(() => {
        loadDevices();
        
        // Poll for status updates every 5 seconds
        const interval = setInterval(() => {
            loadDevices();
        }, 5000);

        return () => clearInterval(interval);
    }, []);

    const loadDevices = async () => {
        try {
            const data = await api.getDevices();
            setDevices(data);
        } catch (error) {
            toast.error("Failed to load devices");
        }
    };

    const handleCreate = async (e: React.FormEvent) => {
        e.preventDefault();
        setFormErrors({});
        
        const errors = validateDevice(newDevice);
        if (Object.keys(errors).length > 0) {
            setFormErrors(errors);
            return;
        }

        try {
            await api.createDevice(newDevice);
            setIsCreateOpen(false);
            setNewDevice({ name: '', mac_address: '', ip_address: '', broadcast_addr: '', icon: 'desktop' });
            loadDevices();
            toast.success("Device created");
        } catch (error) {
            toast.error("Failed to create device");
        }
    };

    const handleEditClick = (device: Device) => {
        setFormErrors({});
        setEditingDevice({
            id: device.id,
            name: device.name,
            mac_address: device.mac_address,
            ip_address: device.ip_address || '',
            broadcast_addr: device.broadcast_addr || '',
            icon: device.icon || 'desktop'
        });
        setIsEditOpen(true);
    };

    const handleUpdate = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!editingDevice) return;
        setFormErrors({});

        const errors = validateDevice(editingDevice);
        if (Object.keys(errors).length > 0) {
            setFormErrors(errors);
            return;
        }

        try {
            await api.updateDevice(editingDevice.id, {
                name: editingDevice.name,
                mac_address: editingDevice.mac_address,
                ip_address: editingDevice.ip_address,
                broadcast_addr: editingDevice.broadcast_addr,
                icon: editingDevice.icon
            });
            setIsEditOpen(false);
            setEditingDevice(null);
            loadDevices();
            toast.success("Device updated");
        } catch (error) {
            toast.error("Failed to update device");
        }
    };

    const handleDelete = async (id: number) => {
        if (!confirm('Are you sure?')) return;
        try {
            await api.deleteDevice(id);
            loadDevices();
            toast.success("Device deleted");
        } catch (error) {
            toast.error("Failed to delete device");
        }
    };

    const handleWake = async (id: number) => {
        try {
            await api.wakeDevice(id);
            toast.success("Wake signal sent");
        } catch (error) {
            toast.error("Failed to send wake signal");
        }
    };

    const handleShutdown = async (id: number) => {
        if (!confirm('Are you sure you want to shutdown this device?')) return;
        try {
            await api.shutdownDevice(id);
            toast.success("Shutdown signal sent");
        } catch (error) {
            toast.error("Failed to send shutdown signal");
        }
    };

    const getIcon = (icon?: string) => {
        switch (icon) {
            case 'server': return <Server className="h-6 w-6" />;
            case 'laptop': return <Laptop className="h-6 w-6" />;
            default: return <Monitor className="h-6 w-6" />;
        }
    };

    return (
        <div className="space-y-6">
            <div className="flex justify-between items-center">
                <h2 className="text-3xl font-bold tracking-tight">Devices</h2>
                {user?.role === 'admin' && (
                    <Dialog open={isCreateOpen} onOpenChange={setIsCreateOpen}>
                        <DialogTrigger asChild>
                            <Button className="gap-2">
                                <Plus className="h-4 w-4" />
                                Add Device
                            </Button>
                        </DialogTrigger>
                        <DialogContent>
                            <DialogHeader>
                                <DialogTitle>Add New Device</DialogTitle>
                            </DialogHeader>
                            <form onSubmit={handleCreate} className="space-y-4">
                                <div className="space-y-2">
                                    <Label>Name</Label>
                                    <Input 
                                        value={newDevice.name}
                                        onChange={e => setNewDevice({...newDevice, name: e.target.value})}
                                        className={formErrors.name ? "border-red-500" : ""}
                                    />
                                    {formErrors.name && <p className="text-sm text-red-500">{formErrors.name}</p>}
                                </div>
                                <div className="space-y-2">
                                    <Label>MAC Address</Label>
                                    <Input 
                                        value={newDevice.mac_address}
                                        onChange={e => setNewDevice({...newDevice, mac_address: e.target.value})}
                                        placeholder="AA:BB:CC:DD:EE:FF"
                                        className={formErrors.mac_address ? "border-red-500" : ""}
                                    />
                                    {formErrors.mac_address && <p className="text-sm text-red-500">{formErrors.mac_address}</p>}
                                </div>
                                <div className="space-y-2">
                                    <Label>IP Address (Optional)</Label>
                                    <Input 
                                        value={newDevice.ip_address}
                                        onChange={e => setNewDevice({...newDevice, ip_address: e.target.value})}
                                        placeholder="192.168.1.100"
                                        className={formErrors.ip_address ? "border-red-500" : ""}
                                    />
                                    {formErrors.ip_address && <p className="text-sm text-red-500">{formErrors.ip_address}</p>}
                                </div>
                                <div className="space-y-2">
                                    <Label>Broadcast Address (Optional)</Label>
                                    <Input 
                                        value={newDevice.broadcast_addr}
                                        onChange={e => setNewDevice({...newDevice, broadcast_addr: e.target.value})}
                                        placeholder="255.255.255.255"
                                        className={formErrors.broadcast_addr ? "border-red-500" : ""}
                                    />
                                    {formErrors.broadcast_addr && <p className="text-sm text-red-500">{formErrors.broadcast_addr}</p>}
                                </div>
                                <DialogFooter>
                                    <Button type="submit">Create</Button>
                                </DialogFooter>
                            </form>
                        </DialogContent>
                    </Dialog>
                )}
            </div>

            <Dialog open={isEditOpen} onOpenChange={setIsEditOpen}>
                <DialogContent>
                    <DialogHeader>
                        <DialogTitle>Edit Device</DialogTitle>
                    </DialogHeader>
                    {editingDevice && (
                        <form onSubmit={handleUpdate} className="space-y-4">
                            <div className="space-y-2">
                                <Label>Name</Label>
                                <Input 
                                    value={editingDevice.name}
                                    onChange={e => setEditingDevice({...editingDevice, name: e.target.value})}
                                    className={formErrors.name ? "border-red-500" : ""}
                                />
                                {formErrors.name && <p className="text-sm text-red-500">{formErrors.name}</p>}
                            </div>
                            <div className="space-y-2">
                                <Label>MAC Address</Label>
                                <Input 
                                    value={editingDevice.mac_address}
                                    onChange={e => setEditingDevice({...editingDevice, mac_address: e.target.value})}
                                    placeholder="AA:BB:CC:DD:EE:FF"
                                    className={formErrors.mac_address ? "border-red-500" : ""}
                                />
                                {formErrors.mac_address && <p className="text-sm text-red-500">{formErrors.mac_address}</p>}
                            </div>
                            <div className="space-y-2">
                                <Label>IP Address (Optional)</Label>
                                <Input 
                                    value={editingDevice.ip_address}
                                    onChange={e => setEditingDevice({...editingDevice, ip_address: e.target.value})}
                                    placeholder="192.168.1.100"
                                    className={formErrors.ip_address ? "border-red-500" : ""}
                                />
                                {formErrors.ip_address && <p className="text-sm text-red-500">{formErrors.ip_address}</p>}
                            </div>
                            <div className="space-y-2">
                                <Label>Broadcast Address (Optional)</Label>
                                <Input 
                                    value={editingDevice.broadcast_addr}
                                    onChange={e => setEditingDevice({...editingDevice, broadcast_addr: e.target.value})}
                                    placeholder="255.255.255.255"
                                    className={formErrors.broadcast_addr ? "border-red-500" : ""}
                                />
                                {formErrors.broadcast_addr && <p className="text-sm text-red-500">{formErrors.broadcast_addr}</p>}
                            </div>
                            <DialogFooter>
                                <Button type="submit">Update</Button>
                            </DialogFooter>
                        </form>
                    )}
                </DialogContent>
            </Dialog>

            <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
                {devices.map(device => (
                    <Card key={device.id} className="hover:shadow-md transition-shadow">
                        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                            <CardTitle className="text-sm font-medium">
                                {device.name}
                            </CardTitle>
                            {getIcon(device.icon)}
                        </CardHeader>
                        <CardContent>
                            <div className="text-xs text-muted-foreground mt-2 font-mono">
                                {device.mac_address}
                            </div>
                            {device.ip_address && (
                                <div className="text-xs text-muted-foreground font-mono">
                                    {device.ip_address}
                                </div>
                            )}
                            <div className="mt-4 flex items-center gap-2">
                                <div className={`h-2.5 w-2.5 rounded-full ${device.is_online ? 'bg-green-500' : 'bg-red-500'}`} />
                                <span className="text-sm text-muted-foreground">
                                    {device.is_online ? 'Online' : 'Offline'}
                                </span>
                            </div>
                        </CardContent>
                        <CardFooter className="flex justify-between">
                            <div className="flex gap-2">
                                <Button size="sm" variant="outline" onClick={() => handleWake(device.id)}>
                                    <Power className="mr-2 h-4 w-4 text-green-600" />
                                    Wake
                                </Button>
                                {device.can_shutdown && (
                                    <Button size="sm" variant="outline" onClick={() => handleShutdown(device.id)}>
                                        <Power className="mr-2 h-4 w-4 text-red-600" />
                                        Off
                                    </Button>
                                )}
                            </div>
                            {user?.role === 'admin' && (
                                <div className="flex gap-1">
                                    <Button size="icon" variant="ghost" onClick={() => handleEditClick(device)}>
                                        <Pencil className="h-4 w-4" />
                                    </Button>
                                    <Button size="icon" variant="ghost" className="text-red-500 hover:text-red-600 hover:bg-red-50" onClick={() => handleDelete(device.id)}>
                                        <Trash2 className="h-4 w-4" />
                                    </Button>
                                </div>
                            )}
                        </CardFooter>
                    </Card>
                ))}
            </div>
        </div>
    );
}
