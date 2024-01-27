import { useState } from "react";
import { useStore } from "zustand";
import Navbar from "~/components/navbar";
import { Button } from "~/components/ui/button";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "~/components/ui/card";
import { Input } from "~/components/ui/input";
import { Label } from "~/components/ui/label";
import { Switch } from "~/components/ui/switch";
import { useToast } from "~/components/ui/use-toast";
import ClientOnly from "~/lib/ClientOnly";
import { useLogin, usePlayer } from "~/store/store";
import { type Server, type LoginData, type PlayerData } from "~/types/types";

export default function Settings() {
    const loginData = useStore(useLogin, (state) => (state as { loginData: LoginData })?.loginData);
    const playerData = useStore(usePlayer, (state) => (state as { playerData: PlayerData })?.playerData);

    const [server, setServer] = useState<Server>("en");

    const { toast } = useToast();

    const updateSeqnum = () => {
        const seqnum = (document.getElementById("seqnum") as HTMLInputElement)?.value;
        useLogin.setState({ loginData: { ...loginData, seqnum } });

        toast({
            title: `Successfully updated seqnum value to ${seqnum}.`,
        });
    };

    return (
        <>
            <main>
                <Navbar />
                <ClientOnly>
                    <div className="mt-4 grid gap-6 px-12">
                        <Card>
                            <CardHeader>
                                <CardTitle>Seqnum Value</CardTitle>
                                <CardDescription>Manually update the seqnum value. If you don&apos;t know what this, don&apos;t worry about it.</CardDescription>
                            </CardHeader>
                            <CardContent>
                                <Input type="number" placeholder="Seqnum value..." defaultValue={loginData.seqnum} id="seqnum" />
                            </CardContent>
                            <CardFooter className="border-t p-6">
                                <Button onClick={updateSeqnum}>Update</Button>
                            </CardFooter>
                        </Card>
                        <Card>
                            <CardHeader>
                                <CardTitle>Account Privacy</CardTitle>
                                <CardDescription>Toggle the privacy setting of your account.</CardDescription>
                            </CardHeader>
                            <CardContent>
                                <div className="flex items-center gap-4">
                                    <Switch id="privacy-switch" />
                                    <Label className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70" htmlFor="privacy-switch">
                                        Enable Privacy
                                    </Label>
                                </div>
                            </CardContent>
                            <CardFooter className="border-t p-6">
                                <Button onClick={() => toast({ title: "Uh oh! This feature hasn't been added yet.", description: "If you want to add this feature, you can ask in the myrtle.moe Discord!" })}>Save</Button>
                            </CardFooter>
                        </Card>
                        <Card>
                            <CardHeader>
                                <CardTitle>Delete Account</CardTitle>
                                <CardDescription>Permanently delete your account and all associated data.</CardDescription>
                            </CardHeader>
                            <CardContent>
                                <Button variant="destructive">Delete Account</Button>
                            </CardContent>
                        </Card>
                    </div>
                </ClientOnly>
            </main>
        </>
    );
}
