; AirCard (Windows) installer - Inno Setup 6
; Build: ISCC.exe installer\aircard.iss /DAppVersion=1.2.4 /DExePath=..\src-audit\AirCard-Windows-HEAD\target\release\aircard.exe
#ifndef AppVersion
  #define AppVersion "1.2.4"
#endif
#ifndef ExePath
  #define ExePath "..\src-audit\AirCard-Windows-HEAD\target\release\aircard.exe"
#endif

[Setup]
AppId={{6F3B8C52-2A47-4E0B-9C1D-0A1C0A4D0001}
AppName=AirCard
AppVersion={#AppVersion}
AppPublisher=AirCard contributors (MIT)
AppPublisherURL=https://github.com/Lumid-Off/AirCard-Windows
DefaultDirName={autopf}\AirCard
DefaultGroupName=AirCard
LicenseFile=LICENSE.txt
OutputDir=Output
OutputBaseFilename=AirCard-Setup-{#AppVersion}
Compression=lzma2
SolidCompression=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=admin
WizardStyle=modern
UninstallDisplayIcon={app}\aircard.exe

[Languages]
Name: "spanish"; MessagesFile: "compiler:Languages\Spanish.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "Crear acceso directo en el escritorio"; Flags: unchecked

[Files]
Source: "{#ExePath}"; DestDir: "{app}"; Flags: ignoreversion
Source: "LICENSE.txt"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\AirCard"; Filename: "{app}\aircard.exe"
Name: "{autodesktop}\AirCard"; Filename: "{app}\aircard.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\aircard.exe"; Description: "Abrir AirCard"; Flags: nowait postinstall skipifsilent

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
    if MsgBox('AirCard necesita "Apple Devices" (componente de Apple) para comunicarse con tu iPhone, y no esta instalado.' + #13#10#13#10 +
              'Quieres instalarlo ahora desde Microsoft Store?' + #13#10 +
              '(Tambien sirve iTunes de 64 bits desde apple.com.)',
              mbConfirmation, MB_YESNO) = IDYES then
    begin
      if not Exec('winget.exe',
                  'install --id 9NP83LWLPZ9K --source msstore --accept-package-agreements --accept-source-agreements',
                  '', SW_SHOW, ewWaitUntilTerminated, ResultCode) or (ResultCode <> 0) then
      begin
        MsgBox('No se pudo instalar automaticamente. Abre Microsoft Store, busca "Apple Devices" e instalalo. Luego abre AirCard.',
               mbInformation, MB_OK);
      end;
    end;
  end;
end;
