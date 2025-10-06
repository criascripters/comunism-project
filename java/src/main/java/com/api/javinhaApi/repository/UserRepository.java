package com.api.javinhaApi.repository;

import org.springframework.data.jpa.repository.JpaRepository;
import com.api.javinhaApi.model.User;

public interface UserRepository extends JpaRepository<User, Long> {

}
