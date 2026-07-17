%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
%% uaqthesis.cls                                                   %%
%% version 4.1                                                     %%
%% 2019-05-15                                                      %%
%% author: Giovanni Stilo,                                         %%
%%                                                                 %%
%% This file is the LaTeX class for the thesis style               %%
%% of the "University of L'Aquila"                                 %%
%%                                                                 %%
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%



%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
%% Identification %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

\NeedsTeXFormat{LaTeX2e}[2009/01/01]
\ProvidesClass{uaqthesis}%
[2018/08/07 Univaq - University of L'Aquila thesis class]

\def\UAQ@classVersion{4.1}

%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
%% Options %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

\RequirePackage{xkeyval}
\newlength{\UAQ@binding}
\newif\ifUAQ@nodefaultfont \UAQ@nodefaultfontfalse
\newif\ifUAQ@romandiff \UAQ@romandifffalse
\newif\ifUAQ@customlanguage \UAQ@customlanguagefalse
\newif\ifUAQ@bn \UAQ@bnfalse
\newif\ifUAQ@PhD \UAQ@PhDfalse
\newif\ifUAQ@Specialization \UAQ@Specializationfalse
\newif\ifUAQ@MasterP \UAQ@MasterPfalse
\newif\ifUAQ@MasterS \UAQ@MasterSfalse
\newif\ifUAQ@LaM \UAQ@LaMfalse
\newif\ifUAQ@Lau \UAQ@Laufalse
\newif\ifUAQ@TFA \UAQ@TFAfalse
\newif\ifUAQ@noexaminfo \UAQ@noexaminfofalse
\newif\ifUAQ@femaleauthor \UAQ@femaleauthorfalse
\newif\ifUAQ@overridecourse \UAQ@overridecoursefalse

\newcommand{\UAQ@customlanguage}{}

\define@key[uaq]{}{noexaminfo}[]{\UAQ@noexaminfotrue}
\define@key[uaq]{}{fem}[]{\UAQ@femaleauthortrue}
\define@key[uaq]{}{nodefaultfont}[]{\UAQ@nodefaultfonttrue}
\define@key[uaq]{}{romandiff}[]{\UAQ@romandifftrue}
\define@key[uaq]{}{italian}[]{\UAQ@customlanguagetrue
  \renewcommand{\UAQ@customlanguage}{\UAQ@italian}}
\define@key[uaq]{}{english}[]{\UAQ@customlanguagetrue
  \renewcommand{\UAQ@customlanguage}{\UAQ@english}}
\define@key[uaq]{}{bn}[]{\UAQ@bntrue
  \PassOptionsToPackage{monochrome}{color}
  \PassOptionsToPackage{monochrome}{xcolor}}
\define@key[uaq]{}{PhD}[]{\UAQ@PhDtrue}
\define@key[uaq]{}{Specialization}[]{\UAQ@Specializationtrue}
\define@key[uaq]{}{LaM}[]{\UAQ@LaMtrue}
\define@key[uaq]{}{Lau}[]{\UAQ@Lautrue}
\define@key[uaq]{}{TFA}[]{\UAQ@TFAtrue}
\define@key[uaq]{}{MasterP}[]{\UAQ@MasterPtrue\UAQ@MasterPtrue}
\define@key[uaq]{}{MasterS}[]{\UAQ@MasterStrue\UAQ@MasterStrue}
\define@key[uaq]{}{binding}[0pt]{\setlength{\UAQ@binding}{#1}}
\define@key[uaq]{}{draft}[]{\PassOptionsToClass{draft}{book}}
\define@key[uaq]{}{oneside}[]{\PassOptionsToClass{oneside}{book}}
\define@key[uaq]{}{twoside}[]{}
\DeclareOptionX*{\ClassError{uaqthesis}{Option '\CurrentOption'%
  \MessageBreak is not valid}{Remove '\CurrentOption' %
  from class options}}
\ProcessOptionsX[uaq]<>
\LoadClass[a4paper,11pt]{book}


\ifUAQ@PhD\relax\else
  \ifUAQ@Specialization\relax\else
    \ifUAQ@LaM\relax\else
      \ifUAQ@Lau\relax\else
        \ifUAQ@MasterP\relax\else
          \ifUAQ@MasterS\relax\else
            \ifUAQ@TFA\relax\else
              \ClassError{uaqthesis}{You must specify the PhD, LaM, Lau, MasterP, %
              MasterS, Specialization or TFA option}{}
            \fi
          \fi
        \fi
      \fi
    \fi
  \fi
\fi



%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
%% Page layout %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

\RequirePackage[a4paper]{geometry}
\geometry{textwidth=396pt,
  textheight=641pt,
  marginparsep=12pt,
  bindingoffset=\UAQ@binding,
  vmarginratio=55:89,
  includehead,
  marginparwidth=72pt,
  heightrounded}
  
\if@twoside\geometry{hmarginratio=55:89}\fi
  



%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
%% Some settings%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
\RequirePackage{ifxetex}
\ifxetex
  \RequirePackage{xltxtra}
\else
  \ifUAQ@nodefaultfont\relax\else
    \RequirePackage[T1]{fontenc}
    \RequirePackage{textcomp}
    \RequirePackage{lmodern}
  \fi
\fi
\RequirePackage{caption}
\captionsetup{indention=\parindent,labelfont={small,bf},labelsep=period,%
  tableposition=top,textfont={small}}
\RequirePackage{graphicx}
\RequirePackage{color}
\RequirePackage{booktabs}
\RequirePackage{amsmath}
\RequirePackage{etoolbox}



%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
%% Fancy headers %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

%% this two lines remove the headers in the empty pages
%% (instead of loading the emptypage package)
\def\cleardoublepage{\clearpage\if@twoside \ifodd\c@page\else
\thispagestyle{empty}\hbox{}\newpage\fi\fi}

\RequirePackage{fancyhdr}

%% definition of the main page style
\fancypagestyle{UAQ@mainstyle}{%
\pagestyle{fancy}
\renewcommand{\chaptermark}[1]{\markboth{\ifnum \c@chapter>0 \thechapter.\ ##1 \else ##1 \fi}{}}
\renewcommand{\sectionmark}[1]{\markright{\thesection\ ##1}}
\fancyhf{}
\fancyhead[OR]{\small\bfseries\thepage}
\fancyhead[OL]{\small\bfseries\nouppercase{\rightmark}}
% This check is useful to avoid Warnings with oneside option
\if@twoside
\fancyhead[ER]{\small\bfseries\nouppercase{\leftmark}}
\fancyhead[EL]{\small\bfseries\thepage}
\fi
\renewcommand{\headrulewidth}{0.4pt}
\renewcommand{\footrulewidth}{0pt}}

%% ri-definition of the plain style (first page of the chapters)
\fancypagestyle{plain}{%
\renewcommand{\headrulewidth}{0pt}
\fancyhf{}
%\fancyfoot[C]{\small\scshape\thepage}
\fancyhead[OR]{\small\bfseries\thepage}
% This check is useful to avoid Warnings with oneside option
\if@twoside
\fancyhead[EL]{\small\bfseries\thepage}
\fi
}

\g@addto@macro\frontmatter{\pagestyle{UAQ@mainstyle}}
\g@addto@macro\mainmatter{\pagestyle{UAQ@mainstyle}}

% fancy marginnote

% commands to redefine the marginpar command
% maybe in future I will remove the mnote command?
%\def \@marginparreset {%
%        \reset@font
%        \footnotesize
%        \itshape
%        \@setminipage
%}
%\long\def\@xmpar[#1]#2{%
%  \@savemarbox\@marbox{\raggedleft #1}%
%  \@savemarbox\@currbox{\raggedright #2}%
%  \@xympar}
%\long\def\@ympar#1{%
%  \@savemarbox\@marbox{\raggedleft #1}%
%  \@savemarbox\@currbox{\raggedright #1}%
%  \@xympar}
%  
% simpler version which cannot redefine marginpar. A new command
% (mnote) is needed
\g@addto@macro\@marginparreset{\footnotesize\itshape}
\newcommand{\mnote}[1]{\marginpar[\raggedleft #1]{\raggedright #1}}



%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
%% numbering objects per chapters (e.g. 1.1, 1.2, ...) %%%%%%%%%%%%%%

\numberwithin{equation}{chapter}
\numberwithin{figure}{chapter}
\numberwithin{table}{chapter}



%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
%% redefinition of the \maketitle command to generate the titlepage %

%% GENERAL COMMANDS

\def\UAQ@subtitle{}
\def\UAQ@courseorganizerLabel{}
\def\UAQ@courseLabel{}

\newcommand{\subtitle}[1]{\def\UAQ@subtitle{#1}}
\newcommand{\courseorganizer}[1]{\def\UAQ@courseorganizerLabel{#1}}
\newcommand{\course}[2][]{%
  \def\UAQ@courseLabel{#2}
  \ifstrequal{#1}{override}{\UAQ@overridecoursetrue}{}}

\def\UAQ@copyyear{}
\newcommand{\copyyear}[1]{\def\UAQ@copyyear{#1}}

\def\UAQ@cycle{}
\newcommand{\cycle}[1]{\def\UAQ@cycle{#1}}

\def\UAQ@submitdate{}
\newcommand{\submitdate}[1]{\def\UAQ@submitdate{#1}}
\let\AcademicYear\submitdate

\def\UAQ@IDnumber{}
\newcommand{\IDnumber}[1]{\def\UAQ@IDnumber{#1}}

%% Advisors
\newcount\UAQ@advisorcount
\newtoks\UAQ@advisortoks
\newcommand{\advisor}[1]{% 
  \ifnum\UAQ@advisorcount=\z@ 
    \UAQ@advisortoks={#1}% 
  \else 
    \UAQ@advisortoks=\expandafter{\the\UAQ@advisortoks\\[0.75mm]#1}% 
  \fi 
  \advance\UAQ@advisorcount\@ne}

%% Co-Advisors or External Advisor 
\newcount\UAQ@coadvisorcount
\newtoks\UAQ@coadvisortoks
\newif\ifUAQ@ext \UAQ@extfalse
\newcommand{\coadvisor}[2][]{%
  \ifnum\UAQ@coadvisorcount=\z@ 
    \UAQ@coadvisortoks={#2}% 
  \else 
    \UAQ@coadvisortoks=\expandafter{\the\UAQ@coadvisortoks\\[0.75mm]#2}% 
  \fi 
  \advance\UAQ@coadvisorcount\@ne
  \ifstrequal{#1}{ext}{\UAQ@exttrue}{}}



%% Reviewers
\def\UAQ@ReviewerLabel{}
\newcommand{\reviewerlabel}[1]{\def\UAQ@ReviewerLabel{#1}}
 
\newcount\UAQ@reviewercount
\newtoks\UAQ@reviewertoks
\newcommand{\reviewer}[1]{%
  \ifnum\UAQ@reviewercount=\z@ 
    \UAQ@reviewertoks={#1}% 
  \else 
    \UAQ@reviewertoks=\expandafter{\the\UAQ@reviewertoks\\[0.75mm]#1}% 
  \fi 
  \advance\UAQ@reviewercount\@ne}  
  
  
  
%% Directors
\newcount\UAQ@directorcount
\newtoks\UAQ@directortoks
\newcommand{\director}[1]{% 
  \ifnum\UAQ@directorcount=\z@ 
    \UAQ@directortoks={#1}% 
  \else 
    \UAQ@directortoks=\expandafter{\the\UAQ@directortoks\\[0.75mm]#1}% 
  \fi 
  \advance\UAQ@directorcount\@ne}

%% Tutor (TFA only)
\newcount\UAQ@tutorcount
\newtoks\UAQ@tutortoks
\newcommand{\tutor}[1]{% 
  \ifnum\UAQ@tutorcount=\z@ 
    \UAQ@tutortoks={#1}% 
  \else 
    \UAQ@tutortoks=\expandafter{\the\UAQ@tutortoks\\[0.75mm]#1}% 
  \fi 
  \advance\UAQ@tutorcount\@ne}

%% Tutor Coordinatori (TFA only)
\newcount\UAQ@tutorcoordcount
\newtoks\UAQ@tutorcoordtoks
\newcommand{\tutorcoord}[1]{% 
  \ifnum\UAQ@tutorcoordcount=\z@ 
    \UAQ@tutorcoordtoks={#1}% 
  \else 
    \UAQ@tutorcoordtoks=\expandafter{\the\UAQ@tutorcoordtoks\\[0.75mm]#1}% 
  \fi 
  \advance\UAQ@tutorcoordcount\@ne}

%% Examiners
\newcount\UAQ@examinercount
\newtoks\UAQ@examinertoks
\newcommand{\examiner}[2][]{% 
  \ifnum\UAQ@examinercount=\z@ 
    \UAQ@examinertoks={#2 (\UAQ@chairman\if|#1|\relax\else, #1\fi)}% 
  \else 
    \UAQ@examinertoks=\expandafter{\the\UAQ@examinertoks\\[0.75mm]#2 \if|#1|\relax\else (#1)\fi}% 
  \fi 
  \advance\UAQ@examinercount\@ne}

\def\UAQ@examdate{}
\newcommand{\examdate}[1]{\def\UAQ@examdate{#1}}

\def\UAQ@versiondate{}
\newcommand{\versiondate}[1]{\def\UAQ@versiondate{#1}}

\def\UAQ@authoremail{}
\newcommand{\authoremail}[1]{\def\UAQ@authoremail{#1}}

\def\UAQ@website{}
\newcommand{\website}[1]{\def\UAQ@website{#1}}

\def\UAQ@ISBN{}
\newcommand{\ISBN}[1]{\def\UAQ@ISBN{#1}}

\def\UAQ@copyrightstatement{}
\newcommand{\copyrightstatement}[1]{\def\UAQ@copyrightstatement{#1}}

\def\UAQ@schoolname{}
\newcommand{\schoolname}[1]{\def\UAQ@schoolname{#1}}

\def\UAQ@schooladdress{}
\newcommand{\schooladdress}[1]{\def\UAQ@schooladdress{#1}}

\def\UAQ@schoolwebsite{}
\newcommand{\schoolwebsite}[1]{\def\UAQ@schoolwebsite{#1}}

\def\UAQ@schoolprincipal{}
\newcommand{\schoolprincipal}[1]{\def\UAQ@schoolprincipal{#1}}


%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
%% PhD maketitle

\newcommand{\UAQ@maketitlePhD}{

% Errors:

\if|\UAQ@courseorganizerLabel|\ClassError{uaqthesis}{You must specify the Ph.D. %
  organizer}{Example: \protect\courseorganizer{Scuola di dottorato di ...}}\fi
\if|\UAQ@courseLabel|\ClassError{uaqthesis}{You must specify the Ph.D. %
  course}{Example: \protect\course{Physics}}\fi
\ifUAQ@overridecourse\relax\else\if|\UAQ@cycle|\ClassError{uaqthesis}{You must specify the Ph.D. %
  cycle (cycle)}{Example: \protect\cycle{XXIII} The Ph.D. cycle must be in the roman numbers}\fi\fi
\if|\@submitdate|\ClassError{uaqthesis}{You must specify the %
  thesis submission date}{Example: \protect\submitdate{August 2012} The date must be in the form Month(name) Year(number)}\fi
\if|\UAQ@copyyear|\ClassError{uaqthesis}{You must specify the %
  copyright year}{Example: \protect\copyyear{2012}}\fi
\if|\UAQ@IDnumber|\ClassError{uaqthesis}{You must specify your %
  ID number (matricola)}{Example: \protect\IDnumber{123456}}\fi
\if|\UAQ@authoremail|\ClassError{uaqthesis}{You must specify the email of the author}{%
  Example: \protect\authoremail{author@email.com}}\fi

% Titlepage:

	\begin{titlepage}
	
	\linespread{1}\selectfont

    \newcommand{\UAQ@parbox}[2][]{\hspace{1.72cm}\parbox[t][##1][t]{11.9cm}{##2}}
    \parindent=0pt
    \sffamily

    \vspace*{-10mm}
    \centering
    \ifUAQ@bn
      \includegraphics[width=4cm]{imgs/univaq_coloured.png}
    \else
      \includegraphics[width=4cm]{imgs/univaq_coloured.png}
    \fi

    \nointerlineskip
    \vspace{\stretch{0.7}}\UAQ@parbox{\Huge\textcolor{uaqgreen}{\UAQ@laquilaname}\par}
    
    \vspace{\stretch{0.9}}
    \nointerlineskip

    \UAQ@parbox{\LARGE\textcolor{uaqgreen}{\@title}\par}
	
	\if|\UAQ@subtitle|
	  \relax
	\else
	  \vspace{0.4cm}
	  \UAQ@parbox{\linespread{1.2}\selectfont\textcolor{uaqgreen}{\large\UAQ@subtitle}\par}
	\fi

    \vspace{\stretch{1}}

    \UAQ@parbox[3cm]{%
    \textcolor{uaqgreen}{\UAQ@courseorganizerLabel\\[2mm]
    \ifUAQ@overridecourse\UAQ@courseLabel\else Dottorato di Ricerca in \UAQ@courseLabel\ -- \UAQ@cycle\ Ciclo\fi}}

    \UAQ@parbox[3cm]{%
    \ifUAQ@femaleauthor
      \UAQ@CandidateLabelFemale\\[2mm]
    \else
      \UAQ@CandidateLabel\\[2mm]
    \fi
    \@author\\[0.75mm]
    \UAQ@IDnumberLabel\ \UAQ@IDnumber}

    
    
    
    \UAQ@parbox[4.5cm]{%
    \makebox[11.9cm][l]{%
      \parbox[t]{5.9cm}{%
        \ifcase\UAQ@advisorcount
          \ClassError{uaqthesis}{You must specify at least one advisor in the preamble}%
          {E.g. \protect\advisor{Prof. Name Surname}}
        \or
          \UAQ@ThesisAdvisorLabel\\[2mm]
        \else
          \UAQ@ThesisAdvisorsLabel\\[2mm]
        \fi
        \the\UAQ@advisortoks}
        \hspace{1.5cm}
        \parbox[t]{5.9cm}{%
        \ifcase\UAQ@coadvisorcount
          \relax
        \or
          \ifUAQ@ext\UAQ@ThesisExtAdvisorLabel\else\UAQ@ThesisCoAdvisorLabel\fi\\[2mm]
        \else
          \ifUAQ@ext\UAQ@ThesisExtAdvisorsLabel\else\UAQ@ThesisCoAdvisorsLabel\fi\\[2mm]
        \fi
        \the\UAQ@coadvisortoks}
    }}

    \UAQ@parbox{\UAQ@submitdate}
    
	\end{titlepage}
\clearpage
\UAQ@composebacktitlepage
}


%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
%% Specialization maketitle

\newcommand{\UAQ@maketitleSpecialization}{

% Errors:

\if|\UAQ@courseorganizerLabel|\ClassError{uaqthesis}{You must specify the Specialization course %
  organizer}{Example: \protect\courseorganizer{Faculty of ...}}\fi
\if|\UAQ@courseLabel|\ClassError{uaqthesis}{You must specify the %
  course}{Example: \protect\course{Fisica}}\fi
\if|\UAQ@submitdate|\ClassError{uaqthesis}{You must specify the Academic %
  Year}{Example: \protect\submitdate{2011/2012}}\fi
\if|\UAQ@IDnumber|\ClassError{uaqthesis}{You must specify your %
  ID number (matricola)}{E.g. 123456}\fi
\if|\UAQ@copyyear|\ClassError{uaqthesis}{You must specify the %
  copyright year}{Example: \protect\copyyear{2012}}\fi
\if|\UAQ@authoremail|\ClassError{uaqthesis}{You must specify the email of the author}{%
  Example: \protect\authoremail{author@email.com}}\fi

% Titlepage:

	\begin{titlepage}
	
	\linespread{1}\selectfont

    \newcommand{\UAQ@parbox}[2][]{\hspace{1.72cm}\parbox[t][##1][t]{11.9cm}{##2}}
    \parindent=0pt
    \sffamily

    \vspace*{-10mm}
    \centering
    \ifUAQ@bn
      \includegraphics[width=4cm]{imgs/univaq_coloured.png}
    \else
      \includegraphics[width=4cm]{imgs/univaq_coloured.png}
    \fi

    \nointerlineskip
    \vspace{\stretch{0.7}}\UAQ@parbox{\Huge\textcolor{uaqgreen}{\UAQ@laquilaname}\par}
    
    \vspace{\stretch{0.9}}
    \nointerlineskip

    \UAQ@parbox{\LARGE\textcolor{uaqgreen}{\@title}\par}
	
	\if|\UAQ@subtitle|
	  \relax
	\else
	  \vspace{0.4cm}
	  \UAQ@parbox{\linespread{1.2}\selectfont\textcolor{uaqgreen}{\large\UAQ@subtitle}\par}
	\fi
	
    \vspace{\stretch{0.9}}

    \UAQ@parbox[3cm]{%
    \textcolor{uaqgreen}{\UAQ@courseorganizerLabel\\[2mm]
    \ifUAQ@overridecourse\UAQ@courseLabel\else Corso di specializzazione in \UAQ@courseLabel\fi}}

    \UAQ@parbox[3cm]{%
    \ifUAQ@femaleauthor
      \UAQ@CandidateLabelFemale\\[2mm]
    \else
      \UAQ@CandidateLabel\\[2mm]
    \fi
    \@author\\[0.75mm]
    \UAQ@IDnumberLabel\ \UAQ@IDnumber}

    \UAQ@parbox[4.5cm]{%
    \makebox[11.9cm][l]{%
      \parbox[t]{5.9cm}{%
        \ifcase\UAQ@advisorcount
          \ClassError{uaqthesis}{You must specify at least one advisor in the preamble}%
          {E.g. \protect\advisor{Prof. Name Surname}}
        \or
          \UAQ@ThesisAdvisorLabel\\[2mm]
        \else
          \UAQ@ThesisAdvisorsLabel\\[2mm]
        \fi
        \the\UAQ@advisortoks}
        \hspace{1.5cm}
        \parbox[t]{5.9cm}{%
        \ifcase\UAQ@directorcount
          \relax
        \or
          \UAQ@ThesisDirectorLabel\\[2mm]
        \else
          \UAQ@ThesisDirectorsLabel\\[2mm]
        \fi
        \the\UAQ@directortoks}
    }
    
    \vspace{1cm}
    
    \makebox[11.9cm][l]{%
          \parbox[t]{5.9cm}{%
            \ifcase\UAQ@coadvisorcount
            \or
              \ifUAQ@ext\UAQ@ThesisExtAdvisorLabel\else\UAQ@ThesisCoAdvisorLabel\fi\\[2mm]
			\else
			  \ifUAQ@ext\UAQ@ThesisExtAdvisorsLabel\else\UAQ@ThesisCoAdvisorsLabel\fi\\[2mm]
            \fi
            \the\UAQ@coadvisortoks}
        }}

    \UAQ@parbox{\UAQ@AcademicYearLabel\ \UAQ@submitdate}
    
	\end{titlepage}
\clearpage
\UAQ@composebacktitlepage
}
%% Laurea and Laurea Magistrale maketitle

\newcommand{\UAQ@maketitleLaurea}{

% Errors:

\if|\UAQ@courseorganizerLabel|\ClassError{uaqthesis}{You must specify the degree %
  organizer}{Example: \protect\courseorganizer{Faculty of ...}}\fi
\if|\UAQ@courseLabel|\ClassError{uaqthesis}{You must specify the %
    course}{Example: \protect\course{Fisica}}\fi
\if|\UAQ@submitdate|\ClassError{uaqthesis}{You must specify the Academic %
  Year}{Example: \protect\submitdate{2011/2012}}\fi
\if|\UAQ@IDnumber|\ClassError{uaqthesis}{You must specify your %
  ID number (matricola)}{E.g. 123456}\fi
\if|\UAQ@copyyear|\ClassError{uaqthesis}{You must specify the %
  copyright year}{Example: \protect\copyyear{2012}}\fi
\if|\UAQ@authoremail|\ClassError{uaqthesis}{You must specify the email of the author}{%
  Example: \protect\authoremail{author@email.com}}\fi

% Titlepage:

	\begin{titlepage}
	
	\linespread{1}\selectfont

    \newcommand{\UAQ@parbox}[2][]{\hspace{1.72cm}\parbox[t][##1][t]{11.9cm}{##2}}
    \parindent=0pt
    \sffamily

    \vspace*{-10mm}
    \centering
    \ifUAQ@bn
      \includegraphics[width=4cm]{imgs/univaq_coloured.png}
    \else
      \includegraphics[width=4cm]{imgs/univaq_coloured.png}
    \fi

    \nointerlineskip
    \vspace{\stretch{0.7}}\UAQ@parbox{\Huge\textcolor{uaqgreen}{\UAQ@laquilaname}\par}

    \vspace{\stretch{0.9}}
    \nointerlineskip

    \UAQ@parbox{\LARGE\textcolor{uaqgreen}{\@title}\par}
	
	\if|\UAQ@subtitle|
	  \relax
	\else
	  \vspace{0.4cm}
	  \UAQ@parbox{\linespread{1.2}\selectfont\textcolor{uaqgreen}{\large\UAQ@subtitle}\par}
	\fi
	
    \vspace{\stretch{0.9}}

    \UAQ@parbox[3cm]{%
    \textcolor{uaqgreen}{\UAQ@courseorganizerLabel\\[2mm]
    \ifUAQ@overridecourse\UAQ@courseLabel\else Corso di Laurea \ifUAQ@LaM Magistrale \fi in \UAQ@courseLabel\fi}}

    \UAQ@parbox[3cm]{%
    \ifUAQ@femaleauthor
      \UAQ@CandidateLabelFemale\\[2mm]
    \else
      \UAQ@CandidateLabel\\[2mm]
    \fi
    \@author\\[0.75mm]
    \UAQ@IDnumberLabel\ \UAQ@IDnumber}

    \UAQ@parbox[4.5cm]{%
    \makebox[11.9cm][l]{%
      \parbox[t]{5.9cm}{%
        \ifcase\UAQ@advisorcount
          \ClassError{uaqthesis}{You must specify at least one advisor in the preamble}%
          {E.g. \protect\advisor{Prof. Name Surname}}
        \or
          \UAQ@ThesisAdvisorLabel\\[2mm]
        \else
          \UAQ@ThesisAdvisorsLabel\\[2mm]
        \fi
        \the\UAQ@advisortoks}
        \hspace{1.5cm}
        \parbox[t]{5.9cm}{%
        \ifcase\UAQ@coadvisorcount
          \relax
        \or
          \ifUAQ@ext\UAQ@ThesisExtAdvisorLabel\else\UAQ@ThesisCoAdvisorLabel\fi\\[2mm]
        \else
          \ifUAQ@ext\UAQ@ThesisExtAdvisorsLabel\else\UAQ@ThesisCoAdvisorsLabel\fi\\[2mm]
        \fi
        \the\UAQ@coadvisortoks}
    }}

    \UAQ@parbox{\UAQ@AcademicYearLabel\ \UAQ@submitdate}
    
	\end{titlepage}
\clearpage
\UAQ@composebacktitlepage
}


%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
%% Master di primo livello and
%% Master di secondo livello maketitle

\newcommand{\UAQ@maketitleMaster}{

% Errors:

\if|\UAQ@courseorganizerLabel|\ClassError{uaqthesis}{You must specify the degree %
  organizer}{Example: \protect\courseorganizer{Faculty of ...}}\fi
\if|\UAQ@courseLabel|\ClassError{uaqthesis}{You must specify the %
  course}{Example: \protect\course{Fisica}}\fi
\if|\UAQ@submitdate|\ClassError{uaqthesis}{You must specify the Academic %
  Year}{Example: \protect\submitdate{2011/2012}}\fi
\if|\UAQ@IDnumber|\ClassError{uaqthesis}{You must specify your %
  ID number (matricola)}{E.g. 123456}\fi
\if|\UAQ@copyyear|\ClassError{uaqthesis}{You must specify the %
  copyright year}{Example: \protect\copyyear{2012}}\fi
\if|\UAQ@authoremail|\ClassError{uaqthesis}{You must specify the email of the author}{%
  Example: \protect\authoremail{author@email.com}}\fi

% Titlepage:

	\begin{titlepage}
	
	\linespread{1}\selectfont

    \newcommand{\UAQ@parbox}[2][]{\hspace{1.72cm}\parbox[t][##1][t]{11.9cm}{##2}}
    \parindent=0pt
    \sffamily

    \vspace*{-10mm}
    \centering
    \ifUAQ@bn
      \includegraphics[width=4cm]{imgs/univaq_coloured.png}
    \else
      \includegraphics[width=4cm]{imgs/univaq_coloured.png}
    \fi
    
    \nointerlineskip
    \vspace{\stretch{0.7}}\UAQ@parbox{\Huge\textcolor{uaqgreen}{\UAQ@laquilaname}\par}

    \vspace{\stretch{0.9}}
    \nointerlineskip

    \UAQ@parbox{\LARGE\textcolor{uaqgreen}{\@title}\par}
	
	\if|\UAQ@subtitle|
	  \relax
	\else
	  \vspace{0.4cm}
	  \UAQ@parbox{\linespread{1.2}\selectfont\textcolor{uaqgreen}{\large\UAQ@subtitle}\par}
	\fi
	
    \vspace{\stretch{0.9}}

    \UAQ@parbox[3cm]{%
    \textcolor{uaqgreen}{\UAQ@courseorganizerLabel\\[2mm]
    \ifUAQ@overridecourse\UAQ@courseLabel\else Master di \ifUAQ@MasterP primo \fi \ifUAQ@MasterS secondo \fi livello in \UAQ@courseLabel\fi}}

    \UAQ@parbox[3cm]{%
    \ifUAQ@femaleauthor
      \UAQ@CandidateLabelFemale\\[2mm]
    \else
      \UAQ@CandidateLabel\\[2mm]
    \fi
    \@author\\[0.75mm]
    \UAQ@IDnumberLabel\ \UAQ@IDnumber}

    \UAQ@parbox[4.5cm]{%
    \makebox[11.9cm][l]{%
      \parbox[t]{5.9cm}{%
        \ifcase\UAQ@advisorcount
          \ClassError{uaqthesis}{You must specify at least one advisor in the preamble}%
          {E.g. \protect\advisor{Prof. Name Surname}}
        \or
          \UAQ@ThesisAdvisorLabel\\[2mm]
        \else
          \UAQ@ThesisAdvisorsLabel\\[2mm]
        \fi
        \the\UAQ@advisortoks}
        \hspace{1.5cm}
        \parbox[t]{5.9cm}{%
        \ifcase\UAQ@coadvisorcount
          \relax
        \or
          \ifUAQ@ext\UAQ@ThesisExtAdvisorLabel\else\UAQ@ThesisCoAdvisorLabel\fi\\[2mm]
        \else
          \ifUAQ@ext\UAQ@ThesisExtAdvisorsLabel\else\UAQ@ThesisCoAdvisorsLabel\fi\\[2mm]
        \fi
        \the\UAQ@coadvisortoks}
    }}

    \UAQ@parbox{\UAQ@AcademicYearLabel\ \UAQ@submitdate}
    
	\end{titlepage}
\clearpage
\UAQ@composebacktitlepage
}


%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
%% Tirocinio Formativo Attivo (TFA) maketitle

\newcommand{\UAQ@maketitleTFA}{

% Errors:

\if|\UAQ@courseorganizerLabel|\ClassError{uaqthesis}{You must specify the degree %
  organizer}{Example: \protect\courseorganizer{Faculty of ...}}\fi
\if|\UAQ@submitdate|\ClassError{uaqthesis}{You must specify the Academic %
  Year}{Example: \protect\submitdate{2011/2012}}\fi
\if|\UAQ@IDnumber|\ClassError{uaqthesis}{You must specify your %
  ID number (matricola)}{E.g. 123456}\fi
\if|\UAQ@copyyear|\ClassError{uaqthesis}{You must specify the %
  copyright year}{Example: \protect\copyyear{2012}}\fi
\if|\UAQ@authoremail|\ClassError{uaqthesis}{You must specify the email of the author}{%
  Example: \protect\authoremail{author@email.com}}\fi

% Titlepage:

	\begin{titlepage}
	
	\linespread{1}\selectfont

    \newcommand{\UAQ@parbox}[2][]{\hspace{1.72cm}\parbox[t][##1][t]{11.9cm}{##2}}
    \parindent=0pt
    \sffamily

    \vspace*{-10mm}
    \centering
    \ifUAQ@bn
      \includegraphics[width=4cm]{imgs/univaq_coloured.png}
    \else
      \includegraphics[width=4cm]{imgs/univaq_coloured.png}
    \fi

    \nointerlineskip
    \vspace{\stretch{0.7}}\UAQ@parbox{\Huge\textcolor{uaqgreen}{\UAQ@laquilaname}\par}
    
    \vspace{\stretch{0.9}}
    \nointerlineskip

    \UAQ@parbox{\LARGE\textcolor{uaqgreen}{\@title}\par}
	
	\if|\UAQ@subtitle|
	  \relax
	\else
	  \vspace{0.4cm}
	  \UAQ@parbox{\linespread{1.2}\selectfont\textcolor{uaqgreen}{\large\UAQ@subtitle}\par}
	\fi
	
    \vspace{\stretch{0.9}}

    \UAQ@parbox[3cm]{%
    \textcolor{uaqgreen}{\UAQ@courseorganizerLabel\\[2mm]
    Tirocinio Formativo Attivo\\[2mm]
    \ifUAQ@overridecourse\UAQ@courseLabel\else Classe \UAQ@courseLabel\fi}}

    \UAQ@parbox[3cm]{%
    \ifUAQ@femaleauthor
      \UAQ@CandidateLabelFemale\\[2mm]
    \else
      \UAQ@CandidateLabel\\[2mm]
    \fi
    \@author\\[0.75mm]
    \UAQ@IDnumberLabel\ \UAQ@IDnumber}

    \UAQ@parbox[4.5cm]{%
    \makebox[11.9cm][l]{%
      \parbox[t]{5.9cm}{%
        \ifcase\UAQ@advisorcount
          \ClassError{uaqthesis}{You must specify at least one advisor in the preamble}%
          {E.g. \protect\advisor{Prof. Name Surname}}
        \or
          \UAQ@ThesisAdvisorLabel\\[2mm]
        \else
          \UAQ@ThesisAdvisorsLabel\\[2mm]
        \fi
        \the\UAQ@advisortoks}
        \hspace{1.5cm}
        \parbox[t]{5.9cm}{%
        \ifcase\UAQ@coadvisorcount
          \relax
        \or
          \ifUAQ@ext\UAQ@ThesisExtAdvisorLabel\else\UAQ@ThesisCoAdvisorLabel\fi\\[2mm]
        \else
          \ifUAQ@ext\UAQ@ThesisExtAdvisorsLabel\else\UAQ@ThesisCoAdvisorsLabel\fi\\[2mm]
        \fi
        \the\UAQ@coadvisortoks}
    }
    
    \vspace{1cm}
    
    \makebox[11.9cm][l]{%
          \parbox[t]{5.9cm}{%
            \ifcase\UAQ@tutorcount
              \ClassError{uaqthesis}{You must specify at least one tutor in the preamble}%
              {E.g. \protect\tutor{Prof. Name Surname}}
            \or
              \UAQ@ThesisTutorLabel\\[2mm]
            \else
              \UAQ@ThesisTutorsLabel\\[2mm]
            \fi
            \the\UAQ@tutortoks}
            \hspace{1.5cm}
            \parbox[t]{5.9cm}{%
            \ifcase\UAQ@tutorcoordcount
              \ClassError{uaqthesis}{You must specify at least one tutor coordinatore in the preamble}%
              {E.g. \protect\tutorcoord{Prof. Name Surname}}
            \or
              \UAQ@ThesisTutorCoordLabel\\[2mm]
            \else
              \UAQ@ThesisTutorCoordsLabel\\[2mm]
            \fi
            \the\UAQ@tutorcoordtoks}
        }}

    \UAQ@parbox{\UAQ@AcademicYearLabel\ \UAQ@submitdate}
    
	\end{titlepage}
\clearpage
\UAQ@composebacktitlepage
}



%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
%% Macro for the back of the title page %%%%%%%%%%%%%%%%%%%%%%%%%%%%%
\def\UAQ@composebacktitlepage{%

{\thispagestyle{empty}
\linespread{1}\selectfont
\setlength{\parindent}{0pt}

\null
\vspace*{1cm}
\ifUAQ@TFA
\UAQ@TFAschoolHeadLabel\\[2mm]
\UAQ@schoolname\\[0.1cm]
\UAQ@schooladdress\\[0.1cm]
\@ifundefined{href}{\UAQ@schoolwebsite}{\href{\UAQ@schoolwebsite}{\UAQ@schoolwebsite}}\\[0.1cm]
\UAQ@TFAschoolPrincipalLabel: \UAQ@schoolprincipal
\vspace*{1.5cm}
\fi

\ifUAQ@noexaminfo\relax
\else
\if?\UAQ@examdate?
  \ifcase\UAQ@examinercount
    \UAQ@ThesisNotDefensedLabel%
  \else
    \ClassError{uaqthesis}{You have specified one or more examiners but not
    the date of the final exam}{E.g. \protect\examdate{17 July 2015}}
  \fi
\else
  \ifcase\UAQ@examinercount
    \ClassError{uaqthesis}{You have specified the final exam date but no examiner}{E.g. \protect\examiner{Prof. Giulio Cesare}}
  \else
    \UAQ@ThesisDefensedLabelA\ \UAQ@examdate\\
    \UAQ@ThesisDefensedLabelB:\\[2mm]
    \the\UAQ@examinertoks%
  \fi
\fi
\fi

\vspace{1cm}

\ifcase\UAQ@reviewercount
  \relax
\else
  \UAQ@ReviewerLabel:\\[2mm]
  \the\UAQ@reviewertoks%
\fi

\vfill

\hrulefill
\vspace{2mm}

\small
\textbf{\def\\{}\def\newline{}\@title} \\[0.75mm]
\ifUAQ@PhD\UAQ@PhDthesistype\fi
\ifUAQ@Specialization\UAQ@Specializationthesistype\fi
\ifUAQ@LaM\UAQ@LaMthesistype\fi
\ifUAQ@Lau\UAQ@Lauthesistype\fi
\ifUAQ@MasterS\UAQ@MasterSthesistype\fi
\ifUAQ@MasterP\UAQ@MasterPthesistype\fi
\ifUAQ@TFA\UAQ@TFAthesistype\fi
. \UAQ@laquilaname \\[0.75mm]
\if|\UAQ@ISBN|\relax\else ISBN: \UAQ@ISBN\fi\\[0.75mm]
\if|\UAQ@copyrightstatement|\copyright\ \UAQ@copyyear\ \@author. \UAQ@allRightsReservedLabel\else\UAQ@copyrightstatement\fi\\[1.5mm]

%Material for which the author is the copyright owner cannot be
%used without the written permission of the author.
%The permission to reproduce copyright protected material does not extend 
%to any material that is copyright of a third party; authorization to
%reproduce such material must be obtained from the copyright owners concerned.
%If you believe that a copyright infringement is present in this thesis,
%please contact the author, providing your name, contact details, full 
%details of the copyright infringement and advise whether you are the copyright
%owner or a representative of the copyright owner.\\

\UAQ@uaqthesisInformationLabel.\\

\if|\UAQ@versiondate|
  \relax%
\else
  \UAQ@versionLabel: \UAQ@versiondate \\[2mm]%
\fi
\if|\UAQ@website|
  \relax%
\else
  \UAQ@websiteLabel: \@ifundefined{href}{\UAQ@website}{\href{\UAQ@website}{\UAQ@website}} \\[2mm]%
\fi
\UAQ@emailLabel: \@ifundefined{href}{\UAQ@authoremail}{\href{mailto:\UAQ@authoremail}{\UAQ@authoremail}}\par

}
\cleardoublepage
}


%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
%% Choosing the right \maketitle according to the thesis type %%%%%%%

\renewcommand{\maketitle}{%
  \ifUAQ@PhD\UAQ@maketitlePhD\fi
  \ifUAQ@Specialization\UAQ@maketitleSpecialization\fi
  \ifUAQ@LaM\UAQ@maketitleLaurea\fi
  \ifUAQ@Lau\UAQ@maketitleLaurea\fi
  \ifUAQ@MasterS\UAQ@maketitleMaster\fi
  \ifUAQ@MasterP\UAQ@maketitleMaster\fi
  \ifUAQ@TFA\UAQ@maketitleTFA\fi}



%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
%% DEDICATION COMMAND %%
\providecommand*{\dedication}[1]{%
\cleardoublepage
\thispagestyle{empty}
\null\vspace{\stretch{0.26}}
\begin{flushright}
\textit{#1}
\end{flushright}
\vspace{\stretch{0.74}}\null}

%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
%% ABSTRACT ENVIRONMENT %%
\newenvironment{abstract}[1][\UAQ@AbstractLabel]
{\cleardoublepage\thispagestyle{plain}
\noindent{\Large\bfseries #1}\par\vspace{0.5cm}\noindent\ignorespaces}
{\null\ignorespacesafterend}

%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
%% ACKNOWLEDGMENTS ENVIRONMENT %%
\newenvironment{acknowledgments}[1][\UAQ@AcknowledgmentsLabel]
{\cleardoublepage\thispagestyle{plain}
\noindent{\Large\bfseries #1}\par\vspace{0.5cm}\itshape\noindent\ignorespaces}
{\null\ignorespacesafterend}


%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
%% LANGUAGE COMMANDS %%

\newcommand{\UAQ@english}{%
  \newcommand{\UAQ@CandidateLabel}{Candidate}
  \newcommand{\UAQ@CandidateLabelFemale}{Candidate}
  \newcommand{\UAQ@IDnumberLabel}{ID number}
  \newcommand{\UAQ@ThesisAdvisorLabel}{Thesis Advisor}
  \newcommand{\UAQ@ThesisAdvisorsLabel}{Thesis Advisors}
  \newcommand{\UAQ@ThesisCoAdvisorLabel}{Co-Advisor}
  \newcommand{\UAQ@ThesisCoAdvisorsLabel}{Co-Advisors}
  \newcommand{\UAQ@ThesisExtAdvisorLabel}{External Advisor}
  \newcommand{\UAQ@ThesisExtAdvisorsLabel}{External Advisors}
  \newcommand{\UAQ@ThesisDirectorLabel}{Director}
  \newcommand{\UAQ@ThesisDirectorsLabel}{Directors}
  \newcommand{\UAQ@ThesisTutorLabel}{Trainee's Tutor}
  \newcommand{\UAQ@ThesisTutorsLabel}{Trainee's Tutors}
  \newcommand{\UAQ@ThesisTutorCoordLabel}{Coordinating Tutor}
  \newcommand{\UAQ@ThesisTutorCoordsLabel}{Coordinating Tutors}
  \newcommand{\UAQ@TFAschoolHeadLabel}{Training done at:}
  \newcommand{\UAQ@TFAschoolPrincipalLabel}{Principal}
  \newcommand{\UAQ@ThesisNotDefensedLabel}{Thesis not yet defended}
  \newcommand{\UAQ@ThesisDefensedLabelA}{Thesis defended on}
  \newcommand{\UAQ@ThesisDefensedLabelB}{in front of a Board of Examiners composed by}
  \newcommand{\UAQ@chairman}{chairman}
  \newcommand{\UAQ@PhDthesistype}{Ph.D. thesis}
  \newcommand{\UAQ@Specializationthesistype}{Specialization thesis}
  \newcommand{\UAQ@LaMthesistype}{Master's thesis}
  \newcommand{\UAQ@Lauthesistype}{Bachelor's thesis}
  \newcommand{\UAQ@MasterPthesistype}{First level master's thesis}
  \newcommand{\UAQ@MasterSthesistype}{Second level master's thesis}
  \newcommand{\UAQ@TFAthesistype}{TFA report}
  \newcommand{\UAQ@laquilaname}{University of L'Aquila}
  \newcommand{\UAQ@versionLabel}{Version}
  \newcommand{\UAQ@websiteLabel}{Website}
  \newcommand{\UAQ@AcknowledgmentsLabel}{Acknowledgments}
  \newcommand{\UAQ@AbstractLabel}{Abstract}
  \newcommand{\UAQ@AcademicYearLabel}{Academic Year}
  \newcommand{\UAQ@allRightsReservedLabel}{All rights reserved}
  \newcommand{\UAQ@uaqthesisInformationLabel}{This thesis has been typeset by \LaTeX\ and the uaqthesis class}
  \newcommand{\UAQ@emailLabel}{Author's email}}

\newcommand{\UAQ@italian}{%
  \newcommand{\UAQ@CandidateLabel}{Candidato}
  \newcommand{\UAQ@CandidateLabelFemale}{Candidata}
  \newcommand{\UAQ@IDnumberLabel}{Matricola}
  \newcommand{\UAQ@ThesisAdvisorLabel}{Relatore}
  \newcommand{\UAQ@ThesisAdvisorsLabel}{Relatori}
  \newcommand{\UAQ@ThesisCoAdvisorLabel}{Correlatore}
  \newcommand{\UAQ@ThesisCoAdvisorsLabel}{Correlatori}
  \newcommand{\UAQ@ThesisExtAdvisorLabel}{Relatore esterno}
  \newcommand{\UAQ@ThesisExtAdvisorsLabel}{Relatori esterni}
  \newcommand{\UAQ@ThesisDirectorLabel}{Direttore}
  \newcommand{\UAQ@ThesisDirectorsLabel}{Direttori}
  \newcommand{\UAQ@ThesisTutorLabel}{Tutor del Tirocinante}
  \newcommand{\UAQ@ThesisTutorsLabel}{Tutor del Tirocinante}
  \newcommand{\UAQ@ThesisTutorCoordLabel}{Tutor Coordinatore}
  \newcommand{\UAQ@ThesisTutorCoordsLabel}{Tutor Coordinatori}
  \newcommand{\UAQ@TFAschoolHeadLabel}{Tirocinio svolto presso:}
  \newcommand{\UAQ@TFAschoolPrincipalLabel}{Dirigente scolastico}
  \newcommand{\UAQ@ThesisNotDefensedLabel}{Tesi non ancora discussa}
  \newcommand{\UAQ@ThesisDefensedLabelA}{Tesi discussa il}
  \newcommand{\UAQ@ThesisDefensedLabelB}{di fronte a una commissione esaminatrice composta da}
  \newcommand{\UAQ@chairman}{presidente}
  \newcommand{\UAQ@PhDthesistype}{Tesi di dottorato}
  \newcommand{\UAQ@Specializationthesistype}{Tesi di Specializzazione}
  \newcommand{\UAQ@LaMthesistype}{Tesi di Laurea Magistrale}
  \newcommand{\UAQ@Lauthesistype}{Tesi di Laurea}
  \newcommand{\UAQ@MasterPthesistype}{Tesi di Master di primo livello}
  \newcommand{\UAQ@MasterSthesistype}{Tesi di Master di secondo livello}
  \newcommand{\UAQ@TFAthesistype}{TFA. Relazione di tirocinio}
  \newcommand{\UAQ@laquilaname}{Universit\`a dell' Aquila}
  \newcommand{\UAQ@versionLabel}{Versione}
  \newcommand{\UAQ@websiteLabel}{Sito web}
  \newcommand{\UAQ@AcknowledgmentsLabel}{Ringraziamenti}
  \newcommand{\UAQ@AbstractLabel}{Sommario}
  \newcommand{\UAQ@AcademicYearLabel}{Anno Accademico}
  \newcommand{\UAQ@allRightsReservedLabel}{Tutti i diritti riservati}
  \newcommand{\UAQ@uaqthesisInformationLabel}{Questa tesi \`e stata composta con \LaTeX\ e la classe uaqthesis}
  \newcommand{\UAQ@emailLabel}{Email dell'autore}}

  
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
%% USEFUL COMMANDS %%

\newcommand*{\UAQ@upbox}[1]{\textrm{\upshape#1}} 
\AtBeginDocument{

  \definecolor{uaqgreen}{RGB}{161,151,65}
  %0.5098039,0.1411765,0.2}

  % scritte in funzione della lingua del documento. Default inglese.
  \ifUAQ@customlanguage
    \UAQ@customlanguage
  \else
    \@ifpackageloaded{babel}{\csname UAQ@\languagename\endcsname}{%
      \@ifpackageloaded{polyglossia}{\csname UAQ@\languagename\endcsname}{\UAQ@english}}
  \fi

  % degree 
  \providecommand{\degree}{\UAQ@upbox{\textdegree}} 
  \providecommand{\g}{\degree} 

  % gradi Celsius
  \providecommand{\celsius}{\ensuremath{\UAQ@upbox{\textdegree}\kern-\scriptspace\mathrm{C}}}
  \ifxetex
    \ProvideTextCommandDefault{\C}{\celsius}
  \else
  
    \providecommand{\C}{\celsius}
  \fi
  
  % Angstrom 
  \providecommand{\angstrom}{\UAQ@upbox{\AA}} 
  \providecommand{\A}{\angstrom} 
  
  % micro 
  \providecommand{\micro}{\UAQ@upbox{\textmu}} 
  
  % Ohm 
  \providecommand{\ohm}{\UAQ@upbox{\textohm}} 
  
  % times (only math) 
  \providecommand{\x}{\times} 
  
  % measurement unit 
  \providecommand*{\un}[1]{\ensuremath{\mathrm{\,#1}}} 

  % Napier's number 
  \providecommand{\eu}{\ensuremath{\mathrm{e}}} 
  
  % imaginary unit 
  \providecommand{\iu}{\ensuremath{\mathrm{i}}} 
  
  % roman subscript 
  \providecommand*{\rb}[1]{\ensuremath{_\mathrm{#1}}} 
  
  % roman superscript 
  \providecommand*{\rp}[1]{\ensuremath{^\mathrm{#1}}} 

  % differential (only math)
  \ifUAQ@romandiff
    \providecommand{\di}{\mathop{}\!\mathrm{d}}
  \else
    \providecommand{\di}{\mathop{}\!d}
  \fi
  
  % derivative and partial derivative (only math)
  % see http://www.guit.sssup.it/phpbb/viewtopic.php?p=42199#42199
  \providecommand{\der}[2]{\begingroup 
    \@tempswafalse\toks@={}\count@=\z@ 
    \@for\next:=#2\do 
      {\expandafter\check@var\next\@nil 
       \advance\count@\der@exp 
       \if@tempswa 
         \toks@=\expandafter{\the\toks@\,}% 
       \else 
         \@tempswatrue 
       \fi 
       \toks@=\expandafter{\the\expandafter\toks@\expandafter\partial\der@var}}%
    \frac{\partial\ifnum\count@=\@ne\else^{\number\count@}\fi#1}{\the\toks@}% 
    \endgroup} 
  \def\check@var{\@ifstar{\mult@var}{\one@var}} 
  \def\mult@var#1#2\@nil{\def\der@var{#2^{#1}}\def\der@exp{#1}} 
  \def\one@var#1\@nil{\def\der@var{#1}\chardef\der@exp\@ne}
  
  \providecommand{\pder}[2]{\begingroup
    \@tempswafalse\toks@={}\count@=\z@
    \@for\next:=#2\do
      {\expandafter\check@var\next\@nil
       \advance\count@\der@exp
       \if@tempswa
         \toks@=\expandafter{\the\toks@\,}%
       \else
         \@tempswatrue
       \fi
       \toks@=\expandafter{\the\expandafter\toks@\expandafter\partial\der@var}}%
    \frac{\partial\ifnum\count@=\@ne\else^{\number\count@}\fi#1}{\the\toks@}%
    \endgroup}
  \def\check@var{\@ifstar{\mult@var}{\one@var}} 
  \def\mult@var#1#2\@nil{\def\der@var{#2^{#1}}\def\der@exp{#1}} 
  \def\one@var#1\@nil{\def\der@var{#1}\chardef\der@exp\@ne}

  % subscript and superscript in text mode
  \DeclareRobustCommand*\textsubscript[1]{\@textsubscript{\selectfont#1}} 
  \def\@textsubscript#1{{\m@th\ensuremath{_{\mbox{\fontsize\sf@size\z@#1}}}}}
  \providecommand*{\tb}[1]{\textsubscript{#1}}
  \providecommand*{\tp}[1]{\textsuperscript{#1}}
}