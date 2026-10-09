; NexusCard installer - Inno Setup 6
; Build: ISCC.exe installer\nexuscard.iss /DAppVersion=1.3.0 /DExePath=..\target\release\nexuscard.exe
#ifndef AppVersion
  #define AppVersion "1.3.0"
#endif
#ifndef ExePath
  #define ExePath "..\target\release\nexuscard.exe"
#endif

[Setup]
AppId={{B1D4E7A2-5C39-4F18-8E6A-2D9F0C7B3A55}
AppName=NexusCard
AppVersion={#AppVersion}
AppPublisher=NexusCard (basado en AirCard, MIT)
AppPublisherURL=https://scalvache.shop/productos/NexusCard/
DefaultDirName={autopf}\NexusCard
DefaultGroupName=NexusCard
LicenseFile=LICENSE.txt
OutputDir=Output
OutputBaseFilename=NexusCard-Setup-{#AppVersion}
Compression=lzma2
SolidCompression=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=admin
WizardStyle=modern
UninstallDisplayIcon={app}\nexuscard.exe

[Languages]
Name: "spanish"; MessagesFile: "compiler:Languages\Spanish.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "Crear acceso directo en el escritorio"; Flags: unchecked

[Files]
Source: "{#ExePath}"; DestDir: "{app}"; Flags: ignoreversion
Source: "LICENSE.txt"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\NexusCard"; Filename: "{app}\nexuscard.exe"
Name: "{autodesktop}\NexusCard"; Filename: "{app}\nexuscard.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\nexuscard.exe"; Description: "Abrir NexusCard"; Flags: nowait postinstall skipifsilent

[Code]
function AppleSupportInstalled(): Boolean;
begin
  Result :=
    FileExists(ExpandConstant('{commoncf64}\Apple\Mobile Device Support\AirTrafficHost.dll')) or
    FileExists(ExpandConstant('{commoncf32}\Apple\Mobile Device Support\AirTrafficHost.dll'));
end;

procedure CurStepChanged(CurStep: TSetupStep);
var
  ResultCode: Integer;
begin
  if (CurStep = ssPostInstall) and (not AppleSupportInstalled()) then
  begin
    if MsgBox('NexusCard necesita "Apple Devices" (componente de Apple) para comunicarse con tu iPhone, y no esta instalado.' + #13#10#13#10 +
              'Quieres instalarlo ahora desde Microsoft Store?' + #13#10 +
              '(Tambien sirve iTunes de 64 bits desde apple.com.)',
              mbConfirmation, MB_YESNO) = IDYES then
    begin
      if not Exec('winget.exe',
                  'install --id 9NP83LWLPZ9K --source msstore --accept-package-agreements --accept-source-agreements',
                  '', SW_SHOW, ewWaitUntilTerminated, ResultCode) or (ResultCode <> 0) then
      begin
        MsgBox('No se pudo instalar automaticamente. Abre Microsoft Store, busca "Apple Devices" e instalalo. Luego abre NexusCard.',
               mbInformation, MB_OK);
      end;
    end;
  end;
end;
