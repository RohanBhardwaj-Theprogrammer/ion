#include "some_code.h"
#include "more_code.h"
#include "util.h"
#include "math_utils.h"
#include "logger.h"
#include "some_message.h"
#include <iostream>
#include <filesystem>
using std::cout;
using std::cin;
using std::endl;
int main(int argc, char** argv ) {

    for(int i = 0 ;i < argc; i++ ) {
        cout<<"Argument "<< i <<" : "<< argv[i] << endl;
    }

    cout<<"============================================="<<endl;
    cout<<"|      current working directory is  " <<  std::filesystem::current_path() <<"      |"<<endl;
    cout<<"============================================="<<endl;

    int mainx ; 
    cin>>mainx ; 
    cout<<endl; 
    cout<<"++++++++++++++++++++++++++++++++++++"<<endl;
    cout<< "The Entered Number is : " << mainx << endl;
    cout<<"++++++++++++++++++++++++++++++++++++"<<endl;
    foo();
    int x = add(2, 3);
    int y = multiply(4, 5);

    log_message("Computation done");
    std::cout<< x + y;

    some_message();
    return 0 ;

}
